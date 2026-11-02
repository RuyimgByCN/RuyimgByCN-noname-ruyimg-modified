//! 仙剑（xianjian）宿主技：弃牌或置牌换取效果（归烬、逐月、水云、忘忧、灵泉、离光、玄墨、探息、破云、械磐、天符）。

use super::super::*;

impl HostMachine {
    pub(super) fn step_xianjian_cards(&mut self, ctx: &mut Ctx<'_>) -> Flow {
        match self.behavior {
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
                        ctx.insert_skill(
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
                            ctx.insert_skill(
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
