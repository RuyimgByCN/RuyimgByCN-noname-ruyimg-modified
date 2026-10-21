use super::super::*;

impl HostMachine {
    pub(super) fn step_skills_card_use(&mut self, ctx: &mut Ctx<'_>) -> Flow {
        match self.behavior {
            HostBehavior::ChangeCardNature => match self.stage {
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
                    self.stage = HostStage::SelectNature;
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectCards,
                            prompt: WUXING_SELECT_CARD,
                            options,
                            min: 1,
                            max: 1,
                            allow_cancel: true,
                        },
                    }
                }
                HostStage::SelectNature => {
                    self.card = match ctx.answer().first() {
                        Some(ChoiceOption::Card(c)) => Some(*c),
                        _ => return Flow::Done, // 弃权
                    };
                    self.stage = HostStage::Act;
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::PickTag,
                            prompt: WUXING_SELECT_NATURE,
                            options: (0..=4).map(ChoiceOption::Tag).collect(),
                            min: 1,
                            max: 1,
                            allow_cancel: true,
                        },
                    }
                }
                HostStage::Act => {
                    let Some(card) = self.card else {
                        return Flow::Done;
                    };
                    if let Some(ChoiceOption::Tag(n)) = ctx.answer().first().copied() {
                        if n <= 4 && ctx.core.player(self.owner).hand.contains(&card) {
                            ctx.core.card_natures.insert(card, n);
                        }
                    }
                    Flow::Done
                }
                _ => Flow::Done,
            },
            HostBehavior::FudichouxinCompare => match self.stage {
                HostStage::CompareTargetPick => {
                    // 目标选牌（拼点牌来自手牌）
                    let Some(t) = self.target else {
                        return Flow::Done;
                    };
                    if ctx.core.player(t).hand.is_empty()
                        || ctx.core.player(self.owner).hand.is_empty()
                    {
                        return Flow::Done;
                    }
                    self.stage = HostStage::ComparePlayerPick;
                    let options: Vec<ChoiceOption> = ctx
                        .core
                        .player(t)
                        .hand
                        .iter()
                        .copied()
                        .map(ChoiceOption::Card)
                        .collect();
                    Flow::WaitChoice {
                        observer: t,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectCards,
                            prompt: WD_FUDI_TARGET_PICK,
                            options,
                            min: 1,
                            max: 1,
                            allow_cancel: false,
                        },
                    }
                }
                HostStage::ComparePlayerPick => {
                    // 消费目标拼点牌 → 挂起己方选牌（点数大者赢）
                    let t_card = match ctx.answer_first() {
                        ChoiceOption::Card(c) => c,
                        _ => return Flow::Done,
                    };
                    self.cards = vec![t_card];
                    self.stage = HostStage::Act;
                    let options: Vec<ChoiceOption> = ctx
                        .core
                        .player(self.owner)
                        .hand
                        .iter()
                        .copied()
                        .map(ChoiceOption::Card)
                        .collect();
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectCards,
                            prompt: WD_FUDI_PLAYER_PICK,
                            options,
                            min: 1,
                            max: 1,
                            allow_cancel: false,
                        },
                    }
                }
                HostStage::Act => {
                    // 消费己方拼点牌：大者赢，赢方获得两张拼点牌
                    let Some(ChoiceOption::Card(p_card)) = ctx.answer().first().copied() else {
                        return Flow::Done;
                    };
                    let t_card = self.cards.first().copied();
                    let pr = ctx.catalog.cards[p_card as usize].rank;
                    if let Some(t_card) = t_card {
                        let tr = ctx.catalog.cards[t_card as usize].rank;
                        if pr > tr {
                            let t = self.target.expect("target set");
                            for c in [p_card, t_card] {
                                ctx.core.player_mut(t).hand.retain(|x| *x != c);
                                ctx.core.player_mut(self.owner).hand.push(c);
                            }
                        }
                    }
                    if let Some(c) = self.card {
                        ctx.to_discard(c);
                        ctx.core.table.retain(|x| *x != c);
                    }
                    Flow::Done
                }
                _ => Flow::Done,
            },
            HostBehavior::TouliangSwap => match self.stage {
                // 三段：己方选牌（SelectCard）→ 目标选牌（SelectNature）→
                // 交换 + 同色摸 1（Act）
                HostStage::SelectCard => {
                    let Some(t) = self.target else {
                        return Flow::Done;
                    };
                    let options: Vec<ChoiceOption> = ctx
                        .core
                        .player(self.owner)
                        .hand
                        .iter()
                        .copied()
                        .map(ChoiceOption::Card)
                        .collect();
                    if options.is_empty() || ctx.core.player(t).hand.is_empty() {
                        return Flow::Done;
                    }
                    self.stage = HostStage::SelectNature;
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectCards,
                            prompt: WD_TOULIANG_OWN,
                            options,
                            min: 1,
                            max: 1,
                            allow_cancel: true,
                        },
                    }
                }
                HostStage::SelectNature => {
                    let own = match ctx.answer_first() {
                        ChoiceOption::Card(c) => c,
                        _ => return Flow::Done,
                    };
                    self.cards = vec![own];
                    self.stage = HostStage::Act;
                    let Some(t) = self.target else {
                        return Flow::Done;
                    };
                    let options: Vec<ChoiceOption> = ctx
                        .core
                        .player(t)
                        .hand
                        .iter()
                        .copied()
                        .map(ChoiceOption::Card)
                        .collect();
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectCards,
                            prompt: WD_TOULIANG_TARGET,
                            options,
                            min: 1,
                            max: 1,
                            allow_cancel: true,
                        },
                    }
                }
                HostStage::Act => {
                    let target_card = match ctx.answer_first() {
                        ChoiceOption::Card(c) => c,
                        _ => return Flow::Done,
                    };
                    let own_card = self.cards.first().copied();
                    let Some(t) = self.target else {
                        return Flow::Done;
                    };
                    let own_color = own_card.map(|c| ctx.catalog.cards[c as usize].suit);
                    let t_color = ctx.catalog.cards[target_card as usize].suit;
                    // 交换：己方失去 own_card 获得目标牌；目标反之
                    if let Some(oc) = own_card {
                        ctx.core.player_mut(self.owner).hand.retain(|x| *x != oc);
                        ctx.core.player_mut(t).hand.push(oc);
                    }
                    ctx.core.player_mut(t).hand.retain(|x| *x != target_card);
                    ctx.core.player_mut(self.owner).hand.push(target_card);
                    // 同色 → 摸 1
                    if own_card.is_some() && own_color == Some(t_color) {
                        ctx.draw_cards(self.owner, 1);
                    }
                    if let Some(c) = self.card {
                        ctx.to_discard(c);
                        ctx.core.table.retain(|x| *x != c);
                    }
                    Flow::Done
                }
                _ => Flow::Done,
            },
            HostBehavior::GeanguanhuoCompare => match self.stage {
                HostStage::CompareTargetPick => {
                    // 目标一选牌（观察者=目标一）
                    let targets = &self.targets;
                    if targets.len() != 2 {
                        return Flow::Done;
                    }
                    let t1 = targets[0];
                    if ctx.core.player(t1).hand.is_empty()
                        || ctx.core.player(targets[1]).hand.is_empty()
                    {
                        return Flow::Done;
                    }
                    self.stage = HostStage::ComparePlayerPick;
                    let options: Vec<ChoiceOption> = ctx
                        .core
                        .player(t1)
                        .hand
                        .iter()
                        .copied()
                        .map(ChoiceOption::Card)
                        .collect();
                    Flow::WaitChoice {
                        observer: t1,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectCards,
                            prompt: YC_GEAN_T1_PICK,
                            options,
                            min: 1,
                            max: 1,
                            allow_cancel: false,
                        },
                    }
                }
                HostStage::ComparePlayerPick => {
                    // 目标二选牌
                    let targets = self.targets.clone();
                    if targets.len() != 2 {
                        return Flow::Done;
                    }
                    self.cards = vec![match ctx.answer_first() {
                        ChoiceOption::Card(c) => c,
                        _ => return Flow::Done,
                    }];
                    self.stage = HostStage::Act;
                    let t2 = targets[1];
                    let options: Vec<ChoiceOption> = ctx
                        .core
                        .player(t2)
                        .hand
                        .iter()
                        .copied()
                        .map(ChoiceOption::Card)
                        .collect();
                    Flow::WaitChoice {
                        observer: t2,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectCards,
                            prompt: YC_GEAN_T2_PICK,
                            options,
                            min: 1,
                            max: 1,
                            allow_cancel: false,
                        },
                    }
                }
                HostStage::Act => {
                    // 结算：大者赢，赢方获得输方一张手牌（确定性首张）；
                    // 平局：目标一弃一张手牌（不计入任何人的收益）
                    let targets = self.targets.clone();
                    if targets.len() != 2 {
                        return Flow::Done;
                    }
                    let (t1, t2) = (targets[0], targets[1]);
                    let Some(ChoiceOption::Card(t2_card)) = ctx.answer().first().copied() else {
                        return Flow::Done;
                    };
                    let Some(t1_card) = self.cards.first().copied() else {
                        return Flow::Done;
                    };
                    let r1 = ctx.catalog.cards[t1_card as usize].rank;
                    let r2 = ctx.catalog.cards[t2_card as usize].rank;
                    if r1 > r2 {
                        if let Some(c) = ctx.core.player(t2).hand.first().copied() {
                            ctx.core.player_mut(t2).hand.retain(|x| *x != c);
                            ctx.core.player_mut(t1).hand.push(c);
                            ctx.log(LogEvent::CardsGiven {
                                from: t2,
                                to: t1,
                                cards: vec![c],
                            });
                        }
                    } else if r2 > r1 {
                        if let Some(c) = ctx.core.player(t1).hand.first().copied() {
                            ctx.core.player_mut(t1).hand.retain(|x| *x != c);
                            ctx.core.player_mut(t2).hand.push(c);
                            ctx.log(LogEvent::CardsGiven {
                                from: t1,
                                to: t2,
                                cards: vec![c],
                            });
                        }
                    } else {
                        // 平局：目标一弃一张手牌
                        if let Some(c) = ctx.core.player(t1).hand.first().copied() {
                            ctx.discard_from_hand(t1, c);
                            ctx.log(LogEvent::CardDiscarded {
                                player: t1,
                                cards: vec![c],
                            });
                        }
                    }
                    if let Some(c) = self.card {
                        ctx.to_discard(c);
                        ctx.core.table.retain(|x| *x != c);
                    }
                    Flow::Done
                }
                _ => Flow::Done,
            },
            HostBehavior::ZhangbaSha => match self.stage {
                // 丈八蛇矛：两张手牌当杀。Stage Act 已由 activate 设置
                // CardsAsk；恢复顺序：选两张 → 选目标 → 发射 CardUsed。
                HostStage::Act => {
                    let options: Vec<ChoiceOption> = ctx
                        .core
                        .player(self.owner)
                        .hand
                        .iter()
                        .copied()
                        .map(ChoiceOption::Card)
                        .collect();
                    if options.len() < 2 {
                        return Flow::Done;
                    }
                    self.stage = HostStage::CardsAsk;
                    return Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectCards,
                            prompt: "zhangba_pick_two",
                            options,
                            min: 2,
                            max: 2,
                            allow_cancel: false,
                        },
                    };
                }
                HostStage::CardsAsk => {
                    let picked: Vec<crate::ids::CardId> = ctx
                        .answer()
                        .iter()
                        .filter_map(|o| match o {
                            ChoiceOption::Card(c) => Some(*c),
                            _ => None,
                        })
                        .collect();
                    if picked.len() != 2 {
                        return Flow::Done;
                    }
                    self.cards = picked;
                    // 目标选择：其他存活且在射程内（丈八射程 3）。
                    let candidates: Vec<ChoiceOption> = ctx
                        .core
                        .alive_players()
                        .filter(|p| {
                            *p != self.owner
                                && ctx.core.computed_distance(ctx.catalog, self.owner, *p)
                                    <= ctx.core.attack_range(ctx.catalog, self.owner)
                        })
                        .map(ChoiceOption::Player)
                        .collect();
                    if candidates.is_empty() {
                        return Flow::Done;
                    }
                    self.stage = HostStage::TargetChoice;
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectPlayers,
                            prompt: "zhangba_sha_target",
                            options: candidates,
                            min: 1,
                            max: 1,
                            allow_cancel: false,
                        },
                    }
                }
                HostStage::TargetChoice => {
                    let (Some(ChoiceOption::Player(target)), [c1, c2]) =
                        (ctx.answer().first(), self.cards.as_slice())
                    else {
                        return Flow::Done;
                    };
                    let target = *target;
                    if !ctx.core.player(target).alive {
                        return Flow::Done;
                    }
                    // 两张手牌弃为转化素材；首张作 CardUsed 载荷牌
                    //（declared Sha，join 判定与伤害归属走杀结算）。
                    ctx.discard_from_hand(self.owner, *c2);
                    let c1_in_hand = ctx.core.player(self.owner).hand.contains(c1);
                    if !c1_in_hand {
                        return Flow::Done;
                    }
                    ctx.core
                        .declared_kinds
                        .insert(*c1, crate::catalog::CardKind::Sha);
                    ctx.emit(crate::event::EventData::CardUsed(
                        crate::machine::card_use_as(
                            *c1,
                            self.owner,
                            vec![target],
                            Some(crate::catalog::CardKind::Sha),
                        ),
                    ));
                    return Flow::Yield; // CardUsed 帧挂起结算（出闪/伤害）
                }
                _ => Flow::Done,
            },
            _ => unreachable!("behavior routed to the wrong step module"),
        }
    }
}
