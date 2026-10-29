use super::super::*;

impl HostMachine {
    pub(super) fn step_xianjian_cards(&mut self, ctx: &mut Ctx<'_>) -> Flow {
        match self.behavior {
            HostBehavior::XianjianDiewu => match self.stage {
                // 蝶舞（U05 xianjian 龙葵，规则 v153）：选 1 张杀+1 目标
                // →直接转移（discard:false give 同口径）→本回合首次摸 1。
                HostStage::SelectCard => {
                    let p = ctx.core.player(self.owner);
                    let options: Vec<ChoiceOption> = p
                        .hand
                        .iter()
                        .copied()
                        .filter(|c| ctx.catalog.card_kind(*c).is_sha_like())
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
                            prompt: WUDIAN_XJ_DIEWU_CARDS,
                            options,
                            min: 1,
                            max: 1,
                            allow_cancel: false,
                        },
                    }
                }
                HostStage::TargetChoice => {
                    // 收「牌选择」应答
                    if self.cards.is_empty() {
                        let cards: Vec<CardId> = ctx
                            .answer()
                            .iter()
                            .filter_map(|o| match o {
                                ChoiceOption::Card(c) => Some(*c),
                                _ => None,
                            })
                            .collect();
                        if cards.len() != 1 {
                            return Flow::Done;
                        }
                        self.cards = cards;
                        let options: Vec<ChoiceOption> = ctx
                            .core
                            .alive_players()
                            .filter(|p| *p != self.owner)
                            .map(ChoiceOption::Player)
                            .collect();
                        if options.is_empty() {
                            return Flow::Done;
                        }
                        Flow::WaitChoice {
                            observer: self.owner,
                            spec: ChoiceSpec {
                                kind: ChoiceKind::SelectPlayers,
                                prompt: WUDIAN_XJ_DIEWU_TARGET,
                                options,
                                min: 1,
                                max: 1,
                                allow_cancel: false,
                            },
                        }
                    } else {
                        // 收「目标选择」应答：转移+首次摸 1
                        let Some(t) = ctx.answer().first().and_then(|o| match o {
                            ChoiceOption::Player(p) => Some(*p),
                            _ => None,
                        }) else {
                            return Flow::Done;
                        };
                        let sel = std::mem::take(&mut self.cards);
                        ctx.give_hand_cards(self.owner, t, &sel);
                        if !ctx.core.turn.diewu_draw_used {
                            ctx.core.turn.diewu_draw_used = true;
                            ctx.draw_cards(self.owner, 1);
                        }
                        Flow::Done
                    }
                }
                _ => Flow::Done,
            },
            HostBehavior::XianjianGuijin => match self.stage {
                // 归烬（U05 xianjian 明绣，规则 v160）：亮牌堆顶 4 张，
                // 逐张交付任意角色（获得者跳过下次摸牌阶段；取消=弃置
                // 剩余；登记差异：上游 chooseCardButton 可批量交付）。
                // 首臂发问（activate 创建首步无应答）。
                HostStage::Act => {
                    if self.target.is_none() {
                        self.target = Some(self.owner); // 装填标记
                        for _ in 0..4 {
                            if ctx.core.deck.is_empty() {
                                let n = ctx.core.discard.len();
                                if n == 0 {
                                    break;
                                }
                                let i = ctx.core.rng.gen_range(n as u32) as usize;
                                let c = ctx.core.discard.remove(i);
                                ctx.core.deck.push(c);
                            }
                            let c = ctx.core.deck.pop().expect("非空");
                            self.cards.push(c);
                        }
                        ctx.core.player_mut(self.owner).guijin_ready_round =
                            ctx.core.turn.round + 3;
                    }
                    if self.cards.is_empty() {
                        return Flow::Done;
                    }
                    let options: Vec<ChoiceOption> =
                        self.cards.iter().copied().map(ChoiceOption::Card).collect();
                    self.stage = HostStage::SelectCard;
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectCards,
                            prompt: WUDIAN_XJ_GUIJIN_CARDS,
                            options,
                            min: 1,
                            max: 1,
                            allow_cancel: true,
                        },
                    }
                }
                HostStage::SelectCard => {
                    // 取消=弃置剩余亮牌收束（上游 result.bool false 同径）
                    let picked = match ctx.answer().first() {
                        Some(ChoiceOption::Card(c)) => Some(*c),
                        _ => None,
                    };
                    let Some(c) = picked else {
                        for r in self.cards.drain(..) {
                            ctx.to_discard(r);
                        }
                        return Flow::Done;
                    };
                    self.cards.retain(|x| *x != c);
                    self.card = Some(c);
                    let options: Vec<ChoiceOption> =
                        ctx.core.alive_players().map(ChoiceOption::Player).collect();
                    self.stage = HostStage::TargetChoice;
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectPlayers,
                            prompt: WUDIAN_XJ_GUIJIN_TARGET,
                            options,
                            min: 1,
                            max: 1,
                            allow_cancel: false,
                        },
                    }
                }
                HostStage::TargetChoice => {
                    let Some(c) = self.card.take() else {
                        return Flow::Done;
                    };
                    if let Some(ChoiceOption::Player(t)) = ctx.answer().first() {
                        let t = *t;
                        if ctx.core.player(t).alive {
                            ctx.core.player_mut(t).hand.push(c);
                            ctx.core.player_mut(t).skip_next_draw = true;
                            ctx.log(LogEvent::CardsGiven {
                                from: self.owner,
                                to: t,
                                cards: vec![c],
                            });
                        } else {
                            ctx.to_discard(c);
                        }
                    }
                    self.stage = HostStage::Act;
                    Flow::Yield
                }
                _ => Flow::Done,
            },
            HostBehavior::XianjianZhuyue => match self.stage {
                // 逐月（U05 xianjian 云天河，规则 v161）：弃一张非基本牌
                //（必选）→ 指定至多两名有牌其他角色 → 各随机弃一张（手
                // 牌优先退随机装备，蓝格怪衣同款登记差异）→ 白名单入
                // TurnState（出牌阶段结束清）。每臂只收一个应答。
                HostStage::SelectCard => {
                    let options: Vec<ChoiceOption> = ctx
                        .core
                        .player(self.owner)
                        .hand
                        .iter()
                        .copied()
                        .filter(|c| yangpijuan_class(ctx.catalog.card_kind(*c)) != Some(0))
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
                            prompt: WUDIAN_XJ_ZHUYUE_CARDS,
                            options,
                            min: 1,
                            max: 1,
                            allow_cancel: false,
                        },
                    }
                }
                HostStage::TargetChoice => {
                    let Some(ChoiceOption::Card(c)) = ctx.answer().first() else {
                        return Flow::Done;
                    };
                    let c = *c;
                    ctx.take_owned_card(self.owner, c);
                    ctx.to_discard(c);
                    ctx.log(LogEvent::CardDiscarded {
                        player: self.owner,
                        cards: vec![c],
                    });
                    let options: Vec<ChoiceOption> = ctx
                        .core
                        .alive_players()
                        .filter(|t| {
                            *t != self.owner
                                && (!ctx.core.player(*t).hand.is_empty()
                                    || !ctx.core.player(*t).equips.is_empty())
                        })
                        .map(ChoiceOption::Player)
                        .collect();
                    if options.is_empty() {
                        return Flow::Done;
                    }
                    let max = options.len().min(2);
                    self.stage = HostStage::Act;
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectPlayers,
                            prompt: WUDIAN_XJ_ZHUYUE_TARGETS,
                            options,
                            min: 1,
                            max,
                            allow_cancel: false,
                        },
                    }
                }
                HostStage::Act => {
                    // 首入（target 为空）：收目标应答装填
                    if self.target.is_none() {
                        self.target = Some(self.owner); // 装填标记
                        let picked: Vec<PlayerId> = ctx
                            .answer()
                            .iter()
                            .filter_map(|o| match o {
                                ChoiceOption::Player(p) => Some(*p),
                                _ => None,
                            })
                            .collect();
                        let picked2 = picked.clone();
                        self.targets = picked;
                        for t in &picked2 {
                            if !ctx.core.turn.zhuyue_targets.contains(t) {
                                ctx.core.turn.zhuyue_targets.push(*t);
                            }
                        }
                    }
                    // 逐目标随机弃一张（存活重查；无牌跳过）
                    let Some(t) = self.targets.first().copied() else {
                        return Flow::Done;
                    };
                    self.targets.remove(0);
                    if !ctx.core.player(t).alive {
                        return Flow::Yield;
                    }
                    if !ctx.core.player(t).hand.is_empty() {
                        let n = ctx.core.player(t).hand.len();
                        let i = ctx.core.rng.gen_range(n as u32) as usize;
                        let c = ctx.core.player(t).hand[i];
                        ctx.discard_from_hand(t, c);
                    } else if !ctx.core.player(t).equips.is_empty() {
                        let n = ctx.core.player(t).equips.len();
                        let j = ctx.core.rng.gen_range(n as u32) as usize;
                        let c = ctx.core.player_mut(t).equips.remove(j);
                        crate::kernel::revoke_equip_skills(ctx.core, ctx.catalog, t, c);
                        ctx.to_discard(c);
                        ctx.log(LogEvent::CardDiscarded {
                            player: t,
                            cards: vec![c],
                        });
                    }
                    Flow::Yield
                }
                _ => Flow::Done,
            },
            HostBehavior::XianjianShuiyun => match self.stage {
                // 水云（U05 xianjian 紫萱，规则 v162）：阶段末从手中选一
                // 张与离场区类型（基本/锦囊/装备，yangpijuan_class 三分
                // 类）不重复的牌置入离场区（≤3）；取消=放弃（上游
                // direct+chooseCard 同口径）。首臂发问。
                HostStage::SelectCard => {
                    let used: Vec<Option<u8>> = ctx
                        .core
                        .player(self.owner)
                        .off_court
                        .iter()
                        .map(|o| yangpijuan_class(ctx.catalog.card_kind(*o)))
                        .collect();
                    let options: Vec<ChoiceOption> = ctx
                        .core
                        .player(self.owner)
                        .hand
                        .iter()
                        .copied()
                        .filter(|c| {
                            let k = yangpijuan_class(ctx.catalog.card_kind(*c));
                            !used.contains(&k)
                        })
                        .map(ChoiceOption::Card)
                        .collect();
                    if options.is_empty() || used.len() >= 3 {
                        return Flow::Done;
                    }
                    self.stage = HostStage::Act;
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectCards,
                            prompt: WUDIAN_XJ_SHUIYUN_CARDS,
                            options,
                            min: 1,
                            max: 1,
                            allow_cancel: true,
                        },
                    }
                }
                HostStage::Act => {
                    if let Some(ChoiceOption::Card(c)) = ctx.answer().first() {
                        let c = *c;
                        if ctx.core.player(self.owner).hand.contains(&c)
                            && ctx.core.player(self.owner).off_court.len() < 3
                        {
                            ctx.hand_to_off_court(self.owner, c);
                        }
                    }
                    Flow::Done
                }
                _ => Flow::Done,
            },
            HostBehavior::XianjianWangyou => match self.stage {
                // 忘忧（U05 xianjian 紫萱，规则 v162）：他人阶段末可弃一
                // 张手牌令本回合受伤标记者（wangyou3_round=当前轮）各摸
                // 1；取消=放弃。首臂发问；逐目标 Yield。
                HostStage::SelectCard => {
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
                    self.stage = HostStage::Act;
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectCards,
                            prompt: WUDIAN_XJ_WANGYOU_CARDS,
                            options,
                            min: 1,
                            max: 1,
                            allow_cancel: true,
                        },
                    }
                }
                HostStage::Act => {
                    if self.target.is_none() {
                        self.target = Some(self.owner); // 装填标记
                        let Some(ChoiceOption::Card(c)) = ctx.answer().first().cloned() else {
                            return Flow::Done; // 取消=放弃
                        };
                        let c = c;
                        ctx.discard_from_hand(self.owner, c);
                        self.targets = ctx
                            .core
                            .seat_order_from(self.owner)
                            .into_iter()
                            .filter(|p| {
                                *p != self.owner
                                    && ctx.core.player(*p).alive
                                    && ctx.core.player(*p).wangyou3_round == ctx.core.turn.round
                            })
                            .collect();
                    }
                    let Some(t) = self.targets.first().copied() else {
                        return Flow::Done;
                    };
                    self.targets.remove(0);
                    if ctx.core.player(t).alive {
                        ctx.draw_cards(t, 1);
                    }
                    Flow::Yield
                }
                _ => Flow::Done,
            },
            HostBehavior::XianjianLingquan => match self.stage {
                // 灵泉（U05 xianjian 小蛮，规则 v163）：觉醒——摸 3+
                // runtime 授水云实例（上游 addSkill("shuiyun") 同口径）
                // +立即执行一次水云置牌（内联询问=上游 createTrigger 同
                // 效）。首臂做效果并发问（类型去重选项，可放弃）。
                HostStage::Act => {
                    ctx.core.player_mut(self.owner).lingquan_awakened = true;
                    ctx.draw_cards(self.owner, 3);
                    if let Some(def) = ctx.catalog.skill_by_name("shuiyun") {
                        ctx.core.skills.insert(
                            (self.owner, def),
                            crate::state::SkillInstance {
                                owner: self.owner,
                                enabled: true,
                                once_used: false,
                                expires_end_of_turn: false,
                                rounds: 0,
                            },
                        );
                    }
                    let used: Vec<Option<u8>> = ctx
                        .core
                        .player(self.owner)
                        .off_court
                        .iter()
                        .map(|o| yangpijuan_class(ctx.catalog.card_kind(*o)))
                        .collect();
                    let options: Vec<ChoiceOption> = ctx
                        .core
                        .player(self.owner)
                        .hand
                        .iter()
                        .copied()
                        // 摸 3 的牌含刚入手牌末——全部参与去重（原恒真
                        // 过滤器由 clippy logic_bug 拦截删除，语义不变）
                        .filter(|c| {
                            let k = yangpijuan_class(ctx.catalog.card_kind(*c));
                            !used.contains(&k)
                        })
                        .map(ChoiceOption::Card)
                        .collect();
                    if options.is_empty() || used.len() >= 3 {
                        return Flow::Done;
                    }
                    self.stage = HostStage::SelectCard;
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectCards,
                            prompt: WUDIAN_XJ_LINGQUAN_CARDS,
                            options,
                            min: 1,
                            max: 1,
                            allow_cancel: true,
                        },
                    }
                }
                HostStage::SelectCard => {
                    if let Some(ChoiceOption::Card(c)) = ctx.answer().first() {
                        let c = *c;
                        if ctx.core.player(self.owner).hand.contains(&c)
                            && ctx.core.player(self.owner).off_court.len() < 3
                        {
                            ctx.hand_to_off_court(self.owner, c);
                        }
                    }
                    Flow::Done
                }
                _ => Flow::Done,
            },
            HostBehavior::XianjianZhaoyao => match self.stage {
                // 照妖（U05 xianjian 南宫煌，规则 v172）：确定性首张拼点
                //（舌战群儒同款登记差异）——赢→摸 2+选两张手牌置牌堆顶
                //（先选在上=后选的先压栈）。输/平→无事。首臂比点并发问。
                HostStage::Act => {
                    let own_r = ctx
                        .core
                        .player(self.owner)
                        .hand
                        .first()
                        .map(|c| ctx.catalog.cards[*c as usize].rank);
                    let t_r = self.target.and_then(|t| {
                        ctx.core
                            .player(t)
                            .hand
                            .first()
                            .map(|c| ctx.catalog.cards[*c as usize].rank)
                    });
                    match (own_r, t_r) {
                        (Some(o), Some(t)) if o > t => {
                            ctx.draw_cards(self.owner, 2);
                            let hand: Vec<CardId> = ctx.core.player(self.owner).hand.clone();
                            if hand.len() < 2 {
                                return Flow::Done;
                            }
                            let options: Vec<ChoiceOption> =
                                hand.iter().copied().map(ChoiceOption::Card).collect();
                            self.stage = HostStage::SelectCard;
                            Flow::WaitChoice {
                                observer: self.owner,
                                spec: ChoiceSpec {
                                    kind: ChoiceKind::SelectCards,
                                    prompt: WUDIAN_XJ_ZHAOYAO_CARDS,
                                    options,
                                    min: 2,
                                    max: 2,
                                    allow_cancel: false,
                                },
                            }
                        }
                        _ => Flow::Done,
                    }
                }
                HostStage::SelectCard => {
                    let picked: Vec<CardId> = ctx
                        .answer()
                        .iter()
                        .filter_map(|o| match o {
                            ChoiceOption::Card(c) => Some(*c),
                            _ => None,
                        })
                        .collect();
                    if picked.len() == 2 {
                        for c in &picked {
                            ctx.take_hand_card(self.owner, *c);
                        }
                        // 先选在上：后选的先压栈
                        for c in picked.iter().rev() {
                            ctx.core.deck.push(*c);
                        }
                    }
                    Flow::Done
                }
                _ => Flow::Done,
            },
            HostBehavior::XianjianLiguang => match self.stage {
                // 离光（U05 xianjian 月琦，规则 v173）：弃一张手牌（必选，
                // 可取消=放弃）→移动场上第一张可移牌至首个合法目的地
                //（确定性，moveCard(true) AI 选取同款登记差异）。装备迁
                // 移 revoke+grant 技能；判定区直接迁移。首臂发问。
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
                    self.stage = HostStage::TargetChoice;
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectCards,
                            prompt: WUDIAN_XJ_LIGUANG_CARDS,
                            options,
                            min: 1,
                            max: 1,
                            allow_cancel: true,
                        },
                    }
                }
                HostStage::TargetChoice => {
                    let Some(ChoiceOption::Card(c)) = ctx.answer().first() else {
                        return Flow::Done; // 取消=放弃
                    };
                    let c = *c;
                    ctx.take_hand_card(self.owner, c);
                    ctx.to_discard(c);
                    ctx.log(LogEvent::CardDiscarded {
                        player: self.owner,
                        cards: vec![c],
                    });
                    // 确定性移动：首张可移牌+首个合法目的地
                    'scan: for src in ctx.core.alive_players().collect::<Vec<_>>() {
                        for eq in ctx.core.player(src).equips.clone() {
                            let slot = ctx.catalog.card_slot_of(ctx.core, eq);
                            let dst = ctx.core.alive_players().find(|dst| {
                                *dst != src
                                    && ctx
                                        .core
                                        .player(*dst)
                                        .equips
                                        .iter()
                                        .all(|o| ctx.catalog.card_slot_of(ctx.core, *o) != slot)
                            });
                            if let Some(dst) = dst {
                                ctx.core.player_mut(src).equips.retain(|x| *x != eq);
                                crate::kernel::revoke_equip_skills(ctx.core, ctx.catalog, src, eq);
                                ctx.core.player_mut(dst).equips.push(eq);
                                crate::kernel::grant_equip_skills(ctx, dst, eq);
                                ctx.log(LogEvent::Equipped {
                                    player: dst,
                                    card: eq,
                                    replaced: None,
                                });
                                break 'scan;
                            }
                        }
                        for jg in ctx.core.player(src).judge.clone() {
                            let k = ctx.catalog.card_kind(jg);
                            let dst = ctx.core.alive_players().find(|dst| {
                                *dst != src
                                    && ctx
                                        .core
                                        .player(*dst)
                                        .judge
                                        .iter()
                                        .all(|o| ctx.catalog.card_kind(*o) != k)
                            });
                            if let Some(dst) = dst {
                                ctx.core.player_mut(src).judge.retain(|x| *x != jg);
                                ctx.core.player_mut(dst).judge.push(jg);
                                break 'scan;
                            }
                        }
                    }
                    Flow::Done
                }
                _ => Flow::Done,
            },
            HostBehavior::XianjianXuanmo => match self.stage {
                // 玄墨（U05 xianjian 夏侯瑾轩，规则 v174）：弃一张三类手
                // 牌置牌堆顶→随机获得牌堆+弃堆中同类型（三分类）实体牌
                // 2 张入手（实体发现基建首例=羊皮卷同款登记差异：上游
                // createCard 凭空造牌，原生实体池发现；不足给余量）。
                // 首臂发问。
                HostStage::SelectCard => {
                    let options: Vec<ChoiceOption> = ctx
                        .core
                        .player(self.owner)
                        .hand
                        .iter()
                        .copied()
                        .filter(|c| yangpijuan_class(ctx.catalog.card_kind(*c)).is_some())
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
                            prompt: WUDIAN_XJ_XUANMO_CARDS,
                            options,
                            min: 1,
                            max: 1,
                            allow_cancel: false,
                        },
                    }
                }
                HostStage::TargetChoice => {
                    let Some(ChoiceOption::Card(c)) = ctx.answer().first() else {
                        return Flow::Done;
                    };
                    let c = *c;
                    let k = yangpijuan_class(ctx.catalog.card_kind(c));
                    ctx.take_hand_card(self.owner, c);
                    ctx.core.deck.push(c); // 置牌堆顶
                    ctx.log(LogEvent::CardPlayed {
                        card: c,
                        player: self.owner,
                        targets: vec![],
                    });
                    // 实体发现：牌堆+弃堆中同类型实体（排除置顶牌）随机
                    // 2 张入手（RNG；不足给余量）
                    let mut pool: Vec<CardId> = ctx
                        .core
                        .deck
                        .iter()
                        .chain(ctx.core.discard.iter())
                        .copied()
                        .filter(|x| *x != c && yangpijuan_class(ctx.catalog.card_kind(*x)) == k)
                        .collect();
                    // RNG 洗牌取前 2
                    let n = pool.len();
                    for i in (1..n).rev() {
                        let j = ctx.core.rng.gen_range(i as u32 + 1) as usize;
                        pool.swap(i, j);
                    }
                    let take = pool.into_iter().take(2).collect::<Vec<_>>();
                    for x in take {
                        if let Some(pos) = ctx.core.deck.iter().position(|y| *y == x) {
                            ctx.core.deck.remove(pos);
                        } else if let Some(pos) = ctx.core.discard.iter().position(|y| *y == x) {
                            ctx.core.discard.remove(pos);
                        }
                        ctx.core.player_mut(self.owner).hand.push(x);
                        ctx.log(LogEvent::CardsDrawn {
                            player: self.owner,
                            cards: vec![x],
                        });
                    }
                    Flow::Done
                }
                _ => Flow::Done,
            },
            HostBehavior::XianjianXtanxi => match self.stage {
                // 探息（U05 xianjian 仙卿，规则 v176）：弃一张手牌→持有
                // 同名牌手牌的其他角色中随机一名→随机获得其一张手牌
                //（亮牌纯表现不迁移；无人持有同名牌则牌照弃）。
                // 首臂发问。
                HostStage::SelectCard => {
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
                    self.stage = HostStage::TargetChoice;
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectCards,
                            prompt: WUDIAN_XJ_XTANXI_CARDS,
                            options,
                            min: 1,
                            max: 1,
                            allow_cancel: true,
                        },
                    }
                }
                HostStage::TargetChoice => {
                    let Some(ChoiceOption::Card(c)) = ctx.answer().first() else {
                        return Flow::Done; // 取消=放弃
                    };
                    let c = *c;
                    let name_kind = ctx.catalog.card_kind(c);
                    ctx.take_hand_card(self.owner, c);
                    ctx.to_discard(c);
                    ctx.log(LogEvent::CardDiscarded {
                        player: self.owner,
                        cards: vec![c],
                    });
                    // 候选：持有同（牌）种手牌的其他角色——同名牌判定按
                    // 牌种（sha/shan 各只有一种实体）
                    let cands: Vec<PlayerId> = ctx
                        .core
                        .alive_players()
                        .filter(|p| {
                            *p != self.owner
                                && ctx
                                    .core
                                    .player(*p)
                                    .hand
                                    .iter()
                                    .any(|h| ctx.catalog.card_kind(*h) == name_kind)
                        })
                        .collect();
                    if cands.is_empty() {
                        return Flow::Done;
                    }
                    let i = ctx.core.rng.gen_range(cands.len() as u32) as usize;
                    let t = cands[i];
                    self.target = Some(t);
                    let hand: Vec<CardId> = ctx.core.player(t).hand.clone();
                    let n = hand.len();
                    let j = ctx.core.rng.gen_range(n as u32) as usize;
                    let taken = hand[j];
                    ctx.core.player_mut(t).hand.retain(|x| *x != taken);
                    ctx.core.player_mut(self.owner).hand.push(taken);
                    ctx.log(LogEvent::CardsGiven {
                        from: t,
                        to: self.owner,
                        cards: vec![taken],
                    });
                    Flow::Done
                }
                _ => Flow::Done,
            },
            HostBehavior::XianjianTianjian => match self.stage {
                // 天剑（U05 xianjian 姜云凡，规则 v178）：弃一张手牌杀
                //（必选）→视为使用万箭齐发（虚拟 AOE 原语：全其他存活）
                // →伤害目标随机弃 he 一张（discard 子技直连=登记差异：
                // 上游为 damageEnd 子技）。
                HostStage::SelectCard => {
                    let options: Vec<ChoiceOption> = ctx
                        .core
                        .player(self.owner)
                        .hand
                        .iter()
                        .copied()
                        .filter(|c| ctx.catalog.card_kind(*c) == crate::catalog::CardKind::Sha)
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
                            prompt: WUDIAN_XJ_TIANJIAN_CARDS,
                            options,
                            min: 1,
                            max: 1,
                            allow_cancel: false,
                        },
                    }
                }
                HostStage::TargetChoice => {
                    let Some(ChoiceOption::Card(c)) = ctx.answer().first() else {
                        return Flow::Done;
                    };
                    let c = *c;
                    ctx.take_hand_card(self.owner, c);
                    ctx.to_discard(c);
                    ctx.log(LogEvent::CardDiscarded {
                        player: self.owner,
                        cards: vec![c],
                    });
                    // 虚拟万箭 AOE
                    let targets: Vec<PlayerId> = ctx
                        .core
                        .seat_order_from(self.owner)
                        .into_iter()
                        .filter(|p| *p != self.owner)
                        .collect();
                    ctx.use_virtual_card(self.owner, CardKind::ArcheryAttack, targets.clone());
                    // 伤害目标随机弃 he 一张（直连）
                    for p in targets {
                        if !ctx.core.player(p).alive {
                            continue;
                        }
                        let mut he: Vec<CardId> = ctx.core.player(p).hand.iter().copied().collect();
                        he.extend(ctx.core.player(p).equips.iter().copied());
                        if he.is_empty() {
                            continue;
                        }
                        let i = ctx.core.rng.gen_range(he.len() as u32) as usize;
                        let dc = he[i];
                        if ctx.core.player(p).hand.contains(&dc) {
                            ctx.discard_from_hand(p, dc);
                        } else {
                            ctx.core.player_mut(p).equips.retain(|x| *x != dc);
                            crate::kernel::revoke_equip_skills(ctx.core, ctx.catalog, p, dc);
                            ctx.to_discard(dc);
                            ctx.log(LogEvent::CardDiscarded {
                                player: p,
                                cards: vec![dc],
                            });
                        }
                    }
                    Flow::Done
                }
                _ => Flow::Done,
            },
            HostBehavior::XianjianPoyun => match self.stage {
                // 破云（U05 xianjian 慕容紫英，规则 v179）：玄砚计数>0
                // 时令受伤目标自选弃 he 1-2 张，计数 -1（direct 无确
                // 认；计数获得路径上游缺失=调试/宿主注入登记差异）。
                // 首臂发问（被弃者自选）。
                HostStage::Act => {
                    let Some(t) = self.target else {
                        return Flow::Done;
                    };
                    let options: Vec<ChoiceOption> = ctx
                        .core
                        .player(t)
                        .hand
                        .iter()
                        .chain(ctx.core.player(t).equips.iter())
                        .copied()
                        .map(ChoiceOption::Card)
                        .collect();
                    if options.is_empty() {
                        return Flow::Done;
                    }
                    let max = options.len().min(2);
                    self.stage = HostStage::TargetChoice;
                    Flow::WaitChoice {
                        observer: t,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectCards,
                            prompt: WUDIAN_XJ_POYUN_CARDS,
                            options,
                            min: 1,
                            max,
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
                    let Some(t) = self.target else {
                        return Flow::Done;
                    };
                    for c in &picked {
                        if ctx.core.player(t).hand.contains(c) {
                            ctx.discard_from_hand(t, *c);
                        } else if ctx.core.player(t).equips.contains(c) {
                            ctx.core.player_mut(t).equips.retain(|x| x != c);
                            crate::kernel::revoke_equip_skills(ctx.core, ctx.catalog, t, *c);
                            ctx.to_discard(*c);
                        }
                    }
                    ctx.core.player_mut(self.owner).xuanning_mark =
                        ctx.core.player(self.owner).xuanning_mark.saturating_sub(1);
                    Flow::Done
                }
                _ => Flow::Done,
            },
            HostBehavior::XianjianQianfang => match self.stage {
                // 千方（U05 xianjian 慕容紫英，规则 v182）：回合开始时
                // 玄砚计数换弃牌（num=3-计数 clamp≥0）后清零，视为对全
                // 部其他角色使用万箭齐发（虚拟 AOE 原语）；被伤敌人每名
                // 紫英摸 1（qianfang_draw 直连=登记差异：上游 damageEnd
                // 逐目标）。首臂发问弃牌（可取消=放弃；num=0 时直接
                // AOE）。
                HostStage::Act => {
                    let mark = ctx.core.player(self.owner).xuanning_mark;
                    let num = 3usize.saturating_sub(mark as usize);
                    let targets: Vec<PlayerId> = ctx
                        .core
                        .seat_order_from(self.owner)
                        .into_iter()
                        .filter(|p| *p != self.owner)
                        .collect();
                    if targets.is_empty() {
                        return Flow::Done;
                    }
                    self.targets = targets.clone();
                    self.cards = vec![CardId::from(num as u32)]; // num 暂存
                    if num == 0 {
                        // 纯 AOE（chooseBool 确认=direct 无询问直接发动）
                        ctx.core.player_mut(self.owner).xuanning_mark = 0;
                        ctx.use_virtual_card(self.owner, CardKind::ArcheryAttack, targets);
                        return Flow::Done;
                    }
                    let options: Vec<ChoiceOption> = ctx
                        .core
                        .player(self.owner)
                        .hand
                        .iter()
                        .chain(ctx.core.player(self.owner).equips.iter())
                        .copied()
                        .map(ChoiceOption::Card)
                        .collect();
                    if options.len() < num {
                        return Flow::Done;
                    }
                    self.stage = HostStage::SelectCard;
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectCards,
                            prompt: WUDIAN_XJ_QIANFANG_CARDS,
                            options,
                            min: num,
                            max: num,
                            allow_cancel: true,
                        },
                    }
                }
                HostStage::SelectCard => {
                    let picked: Vec<CardId> = ctx
                        .answer()
                        .iter()
                        .filter_map(|o| match o {
                            ChoiceOption::Card(c) => Some(*c),
                            _ => None,
                        })
                        .collect();
                    let num =
                        3usize.saturating_sub(ctx.core.player(self.owner).xuanning_mark as usize);
                    if picked.len() != num {
                        return Flow::Done; // 取消=放弃
                    }
                    for c in &picked {
                        if ctx.core.player(self.owner).hand.contains(c) {
                            ctx.discard_from_hand(self.owner, *c);
                        } else {
                            ctx.core.player_mut(self.owner).equips.retain(|x| x != c);
                            crate::kernel::revoke_equip_skills(
                                ctx.core,
                                ctx.catalog,
                                self.owner,
                                *c,
                            );
                            ctx.to_discard(*c);
                        }
                    }
                    ctx.core.player_mut(self.owner).xuanning_mark = 0;
                    let targets = self.targets.clone();
                    ctx.use_virtual_card(self.owner, CardKind::ArcheryAttack, targets.clone());
                    // 被伤敌人每名紫英摸 1（qianfang_draw 直连=登记差异：
                    // 上游 damageEnd 逐目标即时）
                    let hurt: Vec<PlayerId> = targets
                        .into_iter()
                        .filter(|p| ctx.core.player(*p).alive)
                        .collect();
                    let n = hurt.len();
                    ctx.draw_cards(self.owner, n);
                    Flow::Done
                }
                _ => Flow::Done,
            },
            HostBehavior::XianjianXiepan => match self.stage {
                // 械磐（U05 xianjian 居十方，规则 v186）：弃最后一张基本
                // 牌后可弃一张手牌获一件随机装备（可取消=放弃）。
                HostStage::Act => {
                    let options: Vec<ChoiceOption> = ctx
                        .core
                        .player(self.owner)
                        .hand
                        .iter()
                        .copied()
                        .map(ChoiceOption::Card)
                        .collect();
                    self.stage = HostStage::SelectCard;
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectCards,
                            prompt: WUDIAN_XJ_XIEPAN_CARDS,
                            options,
                            min: 1,
                            max: 1,
                            allow_cancel: true,
                        },
                    }
                }
                HostStage::SelectCard => {
                    let Some(ChoiceOption::Card(c)) = ctx.answer().first().copied() else {
                        return Flow::Done; // 取消=放弃
                    };
                    ctx.discard_from_hand(self.owner, c);
                    ctx.log(LogEvent::CardDiscarded {
                        player: self.owner,
                        cards: vec![c],
                    });
                    // 实体发现：牌堆+弃堆装备实体随机 1 张入手（RNG；
                    // 不足无=实体池余量口径）
                    let mut pool: Vec<CardId> = ctx
                        .core
                        .deck
                        .iter()
                        .chain(ctx.core.discard.iter())
                        .copied()
                        .filter(|x| {
                            *x != c && yangpijuan_class(ctx.catalog.card_kind(*x)) == Some(2)
                        })
                        .collect();
                    let n = pool.len();
                    for i in (1..n).rev() {
                        let j = ctx.core.rng.gen_range(i as u32 + 1) as usize;
                        pool.swap(i, j);
                    }
                    if let Some(x) = pool.first().copied() {
                        if let Some(pos) = ctx.core.deck.iter().position(|y| *y == x) {
                            ctx.core.deck.remove(pos);
                        } else if let Some(pos) = ctx.core.discard.iter().position(|y| *y == x) {
                            ctx.core.discard.remove(pos);
                        }
                        ctx.core.player_mut(self.owner).hand.push(x);
                        ctx.log(LogEvent::CardsDrawn {
                            player: self.owner,
                            cards: vec![x],
                        });
                    }
                    Flow::Done
                }
                _ => Flow::Done,
            },
            HostBehavior::XianjianLuanjian => match self.stage {
                // 乱剑（U05 xianjian 长卿，规则 v187）：两张杀当一张杀
                // 使用（虚拟杀=天剑同款弃素材；机器直发不经 PlayCard
                // 校验=无视距离+任意多目标登记差异）。
                HostStage::Act => {
                    let options: Vec<ChoiceOption> = ctx
                        .core
                        .player(self.owner)
                        .hand
                        .iter()
                        .copied()
                        .filter(|c| ctx.catalog.card_kind(*c) == crate::catalog::CardKind::Sha)
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
                            prompt: WUDIAN_XJ_LUANJIAN_CARDS,
                            options,
                            min: 2,
                            max: 2,
                            allow_cancel: true,
                        },
                    }
                }
                HostStage::SelectCard => {
                    let picked: Vec<CardId> = ctx
                        .answer()
                        .iter()
                        .filter_map(|o| match o {
                            ChoiceOption::Card(c) => Some(*c),
                            _ => None,
                        })
                        .collect();
                    if picked.len() != 2 {
                        return Flow::Done; // 取消=放弃
                    }
                    for c in &picked {
                        ctx.discard_from_hand(self.owner, *c);
                    }
                    ctx.log(LogEvent::CardDiscarded {
                        player: self.owner,
                        cards: picked.clone(),
                    });
                    self.cards = picked;
                    // 目标多选（任意名其他存活角色；allow_cancel=false=
                    // 上游 selectTarget [1,∞] 必选，素材已弃不退）
                    let options: Vec<ChoiceOption> = ctx
                        .core
                        .seat_order_from(self.owner)
                        .into_iter()
                        .filter(|p| *p != self.owner && ctx.core.player(*p).alive)
                        .map(ChoiceOption::Player)
                        .collect();
                    if options.is_empty() {
                        return Flow::Done;
                    }
                    let max = options.len();
                    self.stage = HostStage::TargetChoice;
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectPlayers,
                            prompt: WUDIAN_XJ_LUANJIAN_TARGETS,
                            options,
                            min: 1,
                            max,
                            allow_cancel: false,
                        },
                    }
                }
                HostStage::TargetChoice => {
                    let picked: Vec<PlayerId> = ctx
                        .answer()
                        .iter()
                        .filter_map(|o| match o {
                            ChoiceOption::Player(p) => Some(*p),
                            _ => None,
                        })
                        .collect();
                    if picked.is_empty() {
                        return Flow::Done;
                    }
                    // 载体牌锚定（DamageBefore 增伤判定=普通杀/其他虚拟
                    // 技载体不命中）——预读牌堆底=use_virtual_card 载体
                    let carrier = ctx.core.deck.first().copied();
                    ctx.core.turn.luanjian_card = carrier.map(|c| c as u32);
                    ctx.use_virtual_card(self.owner, crate::catalog::CardKind::Sha, picked);
                    Flow::Done
                }
                _ => Flow::Done,
            },
            HostBehavior::XianjianCtianfu => match self.stage {
                // 天符（U05 xianjian 长卿，规则 v188）：弃一张闪置一名
                // 其他角色武将牌上（discard:false=挂附不弃置；离场区承
                // 载+授 ctianfu2 实例+记来源）。
                HostStage::Act => {
                    let options: Vec<ChoiceOption> = ctx
                        .core
                        .player(self.owner)
                        .hand
                        .iter()
                        .copied()
                        .filter(|c| ctx.catalog.card_kind(*c) == crate::catalog::CardKind::Shan)
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
                            prompt: WUDIAN_XJ_CTIANFU_CARD,
                            options,
                            min: 1,
                            max: 1,
                            allow_cancel: true,
                        },
                    }
                }
                HostStage::SelectCard => {
                    let Some(ChoiceOption::Card(c)) = ctx.answer().first().copied() else {
                        return Flow::Done; // 取消=放弃
                    };
                    self.cards = vec![c];
                    // 目标：其他角色且无挂附（上游 filterTarget
                    // !hasSkill("ctianfu2") 同口径）
                    let options: Vec<ChoiceOption> = ctx
                        .core
                        .seat_order_from(self.owner)
                        .into_iter()
                        .filter(|p| {
                            *p != self.owner
                                && ctx.core.player(*p).alive
                                && !ctx.core.skills.contains_key(&(
                                    *p,
                                    ctx.catalog.skill_by_name("ctianfu2").expect("ctianfu2"),
                                ))
                        })
                        .map(ChoiceOption::Player)
                        .collect();
                    if options.is_empty() {
                        // 闪不消耗（未置出）——留手牌
                        return Flow::Done;
                    }
                    self.stage = HostStage::TargetChoice;
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectPlayers,
                            prompt: WUDIAN_XJ_CTIANFU_TARGET,
                            options,
                            min: 1,
                            max: 1,
                            allow_cancel: false,
                        },
                    }
                }
                HostStage::TargetChoice => {
                    let Some(ChoiceOption::Player(t)) = ctx.answer().first().copied() else {
                        return Flow::Done;
                    };
                    let Some(c) = self.cards.first().copied() else {
                        return Flow::Done;
                    };
                    // 挂附：闪离自己手牌 → 置目标武将牌上（离场区）
                    ctx.take_hand_card(self.owner, c);
                    ctx.core.player_mut(t).off_court.push(c);
                    for sub in ["ctianfu2", "ctianfu3"] {
                        if let Some(def) = ctx.catalog.skill_by_name(sub) {
                            ctx.core.skills.insert(
                                (t, def),
                                crate::state::SkillInstance {
                                    owner: t,
                                    enabled: true,
                                    once_used: false,
                                    expires_end_of_turn: false,
                                    rounds: 0,
                                },
                            );
                        }
                    }
                    let pl = ctx.core.player_mut(t);
                    pl.ctianfu_source = Some(self.owner);
                    pl.ctianfu_card = Some(c as u32);
                    Flow::Done
                }
                _ => Flow::Done,
            },
            _ => unreachable!("behavior routed to the wrong step module"),
        }
    }
}
