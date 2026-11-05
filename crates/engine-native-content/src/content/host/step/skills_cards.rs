//! 宿主技能：看牌/拿牌与按伤害取牌（让节、攻心、颂词）。

use super::super::*;

impl HostMachine {
    pub(super) fn step_skills_cards(&mut self, ctx: &mut Ctx<'_>) -> Flow {
        match self.behavior {
            // 让节：按伤害点数逐次——获得牌（摸 1 + 牌堆取指定类型）或移动牌
            HostBehavior::Rangjie => match self.stage {
                HostStage::Act => {
                    if self.cards.is_empty() {
                        // 首次进入：按伤害点数初始化剩余次数
                        let count = ctx.current_damage().map(|d| d.amount).unwrap_or(0);
                        if count == 0 {
                            return Flow::Done;
                        }
                        self.cards = (0..count).map(|_| 0u32).collect();
                    }
                    let movable = ctx
                        .core
                        .players
                        .iter()
                        .any(|p| !p.equips.is_empty() || !p.judge.is_empty());
                    self.stage = HostStage::SelectNature;
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::PickTag,
                            prompt: "rangjie_choice",
                            // 0=获得牌 1=移动牌 2=取消
                            options: if movable {
                                vec![
                                    ChoiceOption::Tag(0),
                                    ChoiceOption::Tag(1),
                                    ChoiceOption::Tag(2),
                                ]
                            } else {
                                vec![ChoiceOption::Tag(0), ChoiceOption::Tag(2)]
                            },
                            min: 1,
                            max: 1,
                            allow_cancel: false,
                        },
                    }
                }
                HostStage::SelectNature => {
                    let Some(ChoiceOption::Tag(choice)) = ctx.answer().first().copied() else {
                        return Flow::Done;
                    };
                    self.cards.pop();
                    match choice {
                        0 => {
                            // 获得牌：摸 1 → 选类型 → 牌堆取一张。R05-C2：
                            // 摸 1 走 R 帧，类型询问在续行段开（同步序＝
                            // 先摸走牌堆顶、再扫描；WaitChoice 期间 R 帧不
                            // 泵，若同步内开询问则恢复步扫到未摸走的顶牌）。
                            ctx.start_draw(self.owner, 1);
                            self.stage = HostStage::DrawSettle;
                            Flow::Yield
                        }
                        1 => {
                            // 移动牌：选场上可见区一张牌
                            let mut options: Vec<ChoiceOption> = Vec::new();
                            for p in 0..ctx.core.players.len() as PlayerId {
                                for c in ctx
                                    .core
                                    .player(p)
                                    .equips
                                    .iter()
                                    .chain(ctx.core.player(p).judge.iter())
                                    .copied()
                                {
                                    options.push(ChoiceOption::Card(c));
                                }
                            }
                            if options.is_empty() {
                                return Flow::Done;
                            }
                            self.stage = HostStage::TargetChoice;
                            Flow::WaitChoice {
                                observer: self.owner,
                                spec: ChoiceSpec {
                                    kind: ChoiceKind::SelectCards,
                                    prompt: "rangjie_move_card",
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
                // R05-C2 续行位：让节「获得牌」摸 1 交付后开类型询问。
                HostStage::DrawSettle => {
                    self.stage = HostStage::SelectCard;
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::PickTag,
                            prompt: "rangjie_type",
                            // 0=基本 1=锦囊 2=装备
                            options: vec![
                                ChoiceOption::Tag(0),
                                ChoiceOption::Tag(1),
                                ChoiceOption::Tag(2),
                            ],
                            min: 1,
                            max: 1,
                            allow_cancel: false,
                        },
                    }
                }
                HostStage::SelectCard => {
                    let Some(ChoiceOption::Tag(t)) = ctx.answer().first().copied() else {
                        return Flow::Done;
                    };
                    let want = match t {
                        0 => 0u8, // 基本
                        1 => 1,   // 锦囊
                        _ => 2,   // 装备
                    };
                    // 实体发现：牌堆中首张该类型牌入手（登记差异：上游
                    // get.cardPile2 随机/搜索面）
                    let found = ctx.core.deck.iter().copied().find(|c| {
                        let kind = ctx.catalog.card_kind(*c);
                        match want {
                            0 => kind.is_basic_kind(),
                            1 => !kind.is_basic_kind() && !kind.is_equip(),
                            _ => kind.is_equip(),
                        }
                    });
                    if let Some(card) = found {
                        ctx.core.deck.retain(|c| *c != card);
                        ctx.core.player_mut(self.owner).hand.push(card);
                        ctx.log(LogEvent::CardsGiven {
                            from: self.owner,
                            to: self.owner,
                            cards: vec![card],
                        });
                    }
                    if self.cards.is_empty() {
                        return Flow::Done;
                    }
                    self.stage = HostStage::Act;
                    Flow::Yield
                }
                HostStage::TargetChoice => {
                    let Some(ChoiceOption::Card(card)) = ctx.answer().first().copied() else {
                        return Flow::Done;
                    };
                    let mut owner_seat = None;
                    for p in 0..ctx.core.players.len() as PlayerId {
                        if ctx.core.player(p).equips.contains(&card)
                            || ctx.core.player(p).judge.contains(&card)
                        {
                            owner_seat = Some(p);
                            break;
                        }
                    }
                    let Some(from) = owner_seat else {
                        return Flow::Done;
                    };
                    // 目的地：其他存活角色（含自己）
                    let options: Vec<ChoiceOption> =
                        ctx.core.alive_players().map(ChoiceOption::Player).collect();
                    self.card = Some(card);
                    self.target = Some(from);
                    self.stage = HostStage::ShuchuiTargetLoop;
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectPlayers,
                            prompt: "rangjie_move_target",
                            options,
                            min: 1,
                            max: 1,
                            allow_cancel: false,
                        },
                    }
                }
                HostStage::ShuchuiTargetLoop => {
                    let Some(card) = self.card else {
                        return Flow::Done;
                    };
                    let Some(from) = self.target else {
                        return Flow::Done;
                    };
                    let Some(ChoiceOption::Player(to)) = ctx.answer().first().copied() else {
                        return Flow::Done;
                    };
                    if !ctx.core.player(to).alive {
                        return Flow::Done;
                    }
                    let was_equip = ctx.core.player(from).equips.contains(&card);
                    if was_equip {
                        ctx.core.player_mut(from).equips.retain(|c| *c != card);
                        crate::kernel::revoke_equip_skills(ctx.core, ctx.catalog, from, card);
                    } else {
                        ctx.core.player_mut(from).judge.retain(|c| *c != card);
                        ctx.note_cards_lost(from, &[card]);
                    }
                    let slot = ctx.catalog.card_slot_of(ctx.core, card);
                    let free = slot.is_some_and(|s| {
                        !ctx.core
                            .player(to)
                            .equips
                            .iter()
                            .any(|c| ctx.catalog.card_slot_of(ctx.core, *c) == Some(s))
                    });
                    if was_equip && free {
                        ctx.core.player_mut(to).equips.push(card);
                        crate::kernel::grant_equip_skills(ctx, to, card);
                    } else {
                        ctx.core.player_mut(to).hand.push(card);
                    }
                    ctx.log(LogEvent::CardsGiven {
                        from,
                        to,
                        cards: vec![card],
                    });
                    if self.cards.is_empty() {
                        return Flow::Done;
                    }
                    self.stage = HostStage::Act;
                    Flow::Yield
                }
                _ => Flow::Done,
            },
            // 攻心：选目标 → 看其手牌选一张红桃 → 弃置或置于牌堆顶
            HostBehavior::Gongxin => match self.stage {
                HostStage::Act => {
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
                            prompt: "gongxin_target",
                            options,
                            min: 1,
                            max: 1,
                            allow_cancel: true,
                        },
                    }
                }
                HostStage::TargetChoice => {
                    let Some(ChoiceOption::Player(target)) = ctx.answer().first().copied() else {
                        return Flow::Done;
                    };
                    if !ctx.core.player(target).alive || ctx.core.player(target).hand.is_empty() {
                        return Flow::Done;
                    }
                    self.target = Some(target);
                    self.stage = HostStage::SelectCard;
                    let options: Vec<ChoiceOption> = ctx
                        .core
                        .player(target)
                        .hand
                        .iter()
                        .copied()
                        .map(ChoiceOption::Card)
                        .collect();
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectCards,
                            prompt: "gongxin_card",
                            options,
                            min: 1,
                            max: 1,
                            allow_cancel: true,
                        },
                    }
                }
                HostStage::SelectCard => {
                    let Some(target) = self.target else {
                        return Flow::Done;
                    };
                    let Some(ChoiceOption::Card(card)) = ctx.answer().first().copied() else {
                        return Flow::Done;
                    };
                    if !ctx.core.player(target).hand.contains(&card) {
                        return Flow::Done;
                    }
                    if ctx.catalog.cards[card as usize].suit != crate::catalog::Suit::Heart {
                        self.stage = HostStage::SelectCard;
                        let options: Vec<ChoiceOption> = ctx
                            .core
                            .player(target)
                            .hand
                            .iter()
                            .copied()
                            .map(ChoiceOption::Card)
                            .collect();
                        return Flow::WaitChoice {
                            observer: self.owner,
                            spec: ChoiceSpec {
                                kind: ChoiceKind::SelectCards,
                                prompt: "gongxin_card",
                                options,
                                min: 1,
                                max: 1,
                                allow_cancel: true,
                            },
                        };
                    }
                    self.card = Some(card);
                    self.stage = HostStage::SelectNature;
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::PickTag,
                            prompt: "gongxin_choice",
                            options: vec![ChoiceOption::Tag(0), ChoiceOption::Tag(1)],
                            min: 1,
                            max: 1,
                            allow_cancel: false,
                        },
                    }
                }
                HostStage::SelectNature => {
                    let Some(target) = self.target else {
                        return Flow::Done;
                    };
                    let Some(card) = self.card else {
                        return Flow::Done;
                    };
                    let Some(ChoiceOption::Tag(choice)) = ctx.answer().first().copied() else {
                        return Flow::Done;
                    };
                    if !ctx.core.player(target).hand.contains(&card) {
                        return Flow::Done;
                    }
                    ctx.log(LogEvent::SkillTriggered {
                        skill: "攻心",
                        owner: self.owner, presentation: None });
                    if choice == 0 {
                        ctx.discard_from_hand(target, card);
                    } else {
                        ctx.take_hand_card(target, card);
                        ctx.core.deck.push(card);
                        ctx.log(LogEvent::CardsGiven {
                            from: target,
                            to: self.owner,
                            cards: vec![card],
                        });
                    }
                    Flow::Done
                }
                _ => Flow::Done,
            },
            // 颂词：选未记录角色 → 手牌多于体力弃 2 否则摸 2 → 记录
            HostBehavior::Songci => match self.stage {
                HostStage::Act => {
                    let recorded = &ctx.core.player(self.owner).songci_targets;
                    let options: Vec<ChoiceOption> = ctx
                        .core
                        .alive_players()
                        .filter(|p| !recorded.contains(p))
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
                            prompt: "songci_target",
                            options,
                            min: 1,
                            max: 1,
                            allow_cancel: false,
                        },
                    }
                }
                HostStage::TargetChoice => {
                    let Some(ChoiceOption::Player(target)) = ctx.answer().first().copied() else {
                        return Flow::Done;
                    };
                    if !ctx.core.player(target).alive {
                        return Flow::Done;
                    }
                    self.target = Some(target);
                    ctx.log(LogEvent::SkillTriggered {
                        skill: "颂词",
                        owner: self.owner, presentation: None });
                    let hand = ctx.core.player(target).hand.len();
                    let hp = ctx.core.player(target).hp.max(0) as usize;
                    if hand > hp {
                        // 手牌多于体力：强制弃 2 张（手/装备；不足则弃全部，
                        // 登记差异）
                        let cards: Vec<CardId> = ctx
                            .core
                            .player(target)
                            .hand
                            .iter()
                            .chain(ctx.core.player(target).equips.iter())
                            .copied()
                            .collect();
                        let need = cards.len().min(2);
                        if need > 0 {
                            self.cards = cards.into_iter().take(need).map(u32::from).collect();
                            self.stage = HostStage::SelectCard;
                            return Flow::WaitChoice {
                                observer: target,
                                spec: ChoiceSpec {
                                    kind: ChoiceKind::SelectCards,
                                    prompt: "songci_discard",
                                    options: self
                                        .cards
                                        .iter()
                                        .map(|c| ChoiceOption::Card(*c as CardId))
                                        .collect(),
                                    min: need,
                                    max: need,
                                    allow_cancel: false,
                                },
                            };
                        }
                    } else {
                        // R05-C2：步末纯摸牌 → R 帧；songci_record 纯状态
                        // 翻转（无日志/帧），次序不可观测。
                        ctx.start_draw(target, 2);
                    }
                    songci_record(ctx, self.owner, target);
                    Flow::Done
                }
                HostStage::SelectCard => {
                    let Some(t) = self.target else {
                        return Flow::Done;
                    };
                    let picked: Vec<CardId> = ctx
                        .answer()
                        .iter()
                        .filter_map(|o| match o {
                            ChoiceOption::Card(c) => Some(*c),
                            _ => None,
                        })
                        .collect();
                    if picked.is_empty() {
                        return Flow::Done;
                    }
                    for c in picked {
                        if ctx.core.player(t).hand.contains(&c) {
                            ctx.discard_from_hand(t, c);
                        } else if ctx.core.player(t).equips.contains(&c) {
                            ctx.core.player_mut(t).equips.retain(|x| *x != c);
                            crate::kernel::revoke_equip_skills(ctx.core, ctx.catalog, t, c);
                            ctx.to_discard(c);
                        }
                    }
                    songci_record(ctx, self.owner, t);
                    Flow::Done
                }
                _ => Flow::Done,
            },
            _ => unreachable!("behavior routed to the wrong step module"),
        }
    }
}
