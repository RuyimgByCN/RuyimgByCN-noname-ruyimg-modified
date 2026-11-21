//! 宿主技能：看牌/拿牌与按伤害取牌（让节、攻心、颂词）。

use super::super::*;

pub(in crate::content::host) fn rangjie_can_move_to(
    core: &crate::state::GameStateCore,
    catalog: &crate::catalog::Catalog,
    from: PlayerId,
    card: CardId,
    to: PlayerId,
) -> bool {
    if to == from || !core.player(to).alive {
        return false;
    }
    if core.player(from).equips.contains(&card) {
        let Some(slot) = catalog.card_slot_of(core, card) else {
            return false;
        };
        return !core
            .player(to)
            .equips
            .iter()
            .any(|&c| catalog.card_slot_of(core, c) == Some(slot));
    }
    if core.player(from).judge.contains(&card) {
        let kind = catalog.card_kind(card);
        return !core
            .player(to)
            .judge
            .iter()
            .any(|&c| catalog.card_kind(c) == kind);
    }
    false
}

pub(in crate::content::host) fn rangjie_movable_cards(
    core: &crate::state::GameStateCore,
    catalog: &crate::catalog::Catalog,
) -> Vec<CardId> {
    let mut out = Vec::new();
    for from in 0..core.players.len() as PlayerId {
        for &c in core
            .player(from)
            .equips
            .iter()
            .chain(core.player(from).judge.iter())
        {
            if core
                .alive_players()
                .any(|to| rangjie_can_move_to(core, catalog, from, c, to))
            {
                out.push(c);
            }
        }
    }
    out
}

pub(in crate::content::host) fn rangjie_move_targets(
    core: &crate::state::GameStateCore,
    catalog: &crate::catalog::Catalog,
    from: PlayerId,
    card: CardId,
) -> Vec<PlayerId> {
    core.alive_players()
        .filter(|&to| rangjie_can_move_to(core, catalog, from, card, to))
        .collect()
}

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
                    let movable = !rangjie_movable_cards(ctx.core, ctx.catalog).is_empty();
                    self.stage = HostStage::SelectNature;
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::PickTag,
                            prompt: "rangjie_choice",
                            // 0=获得牌 1=移动牌 2=取消（按上游仍摸1并尝试移牌，cancel2=空取消才直接结束）
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
                            allow_cancel: true,
                        },
                    }
                }
                HostStage::SelectNature => {
                    if ctx.answer().is_empty() {
                        // cancel2：直接结束不摸牌（史诗卡牌/extension.js:450）
                        self.cards.clear();
                        return Flow::Done;
                    }
                    let Some(ChoiceOption::Tag(choice)) = ctx.answer().first().copied() else {
                        self.cards.clear();
                        return Flow::Done;
                    };
                    self.cards.pop();
                    match choice {
                        0 => {
                            // 获得牌：摸 1 → 选类型 → 牌堆顶起取一张。
                            self.target = None;
                            ctx.start_draw(self.owner, 1);
                            self.stage = HostStage::DrawSettle;
                            Flow::Yield
                        }
                        // `extension.js:450-483`：非 cancel2（含字面「取消」2 与「移动牌」1）均摸 1 并进移牌
                        _ => {
                            self.target = Some(1);
                            ctx.start_draw(self.owner, 1);
                            self.stage = HostStage::DrawSettle;
                            Flow::Yield
                        }
                    }
                }
                // R05-C2 续行位：让节摸 1 交付后按分支开类型询问或移动牌询问。
                HostStage::DrawSettle => {
                    if self.target == Some(1) {
                        self.target = None;
                        let cards = rangjie_movable_cards(ctx.core, ctx.catalog);
                        if cards.is_empty() {
                            if self.cards.is_empty() {
                                return Flow::Done;
                            }
                            self.stage = HostStage::Act;
                            return Flow::Yield;
                        }
                        self.stage = HostStage::TargetChoice;
                        return Flow::WaitChoice {
                            observer: self.owner,
                            spec: ChoiceSpec {
                                kind: ChoiceKind::SelectCards,
                                prompt: "rangjie_move_card",
                                options: cards.into_iter().map(ChoiceOption::Card).collect(),
                                min: 1,
                                max: 1,
                                allow_cancel: false,
                            },
                        };
                    }
                    self.stage = HostStage::SelectCard;
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::PickTag,
                            prompt: "rangjie_type",
                            // 0=基本 1=锦囊 2=装备（可取消跳过取牌并继续下一循环）
                            options: vec![
                                ChoiceOption::Tag(0),
                                ChoiceOption::Tag(1),
                                ChoiceOption::Tag(2),
                            ],
                            min: 1,
                            max: 1,
                            allow_cancel: true,
                        },
                    }
                }
                HostStage::SelectCard => {
                    if let Some(ChoiceOption::Tag(t)) = ctx.answer().first().copied() {
                        let want = match t {
                            0 => 0u8, // 基本
                            1 => 1,   // 锦囊
                            _ => 2,   // 装备
                        };
                        // 实体发现：从牌堆顶（deck 尾部）起扫描首张该类型牌入手（game.js:63187）
                        let found = ctx.core.deck.iter().rev().copied().find(|c| {
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
                    let targets = rangjie_move_targets(ctx.core, ctx.catalog, from, card);
                    if targets.is_empty() {
                        if self.cards.is_empty() {
                            return Flow::Done;
                        }
                        self.stage = HostStage::Act;
                        return Flow::Yield;
                    }
                    self.card = Some(card);
                    self.target = Some(from);
                    self.stage = HostStage::ShuchuiTargetLoop;
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectPlayers,
                            prompt: "rangjie_move_target",
                            options: targets.into_iter().map(ChoiceOption::Player).collect(),
                            min: 1,
                            max: 1,
                            allow_cancel: false,
                        },
                    }
                }
                HostStage::ShuchuiTargetLoop => {
                    let Some(card) = self.card.take() else {
                        return Flow::Done;
                    };
                    let Some(from) = self.target.take() else {
                        return Flow::Done;
                    };
                    let Some(ChoiceOption::Player(to)) = ctx.answer().first().copied() else {
                        return Flow::Done;
                    };
                    if !rangjie_can_move_to(ctx.core, ctx.catalog, from, card, to) {
                        return Flow::Done;
                    }
                    let was_equip = ctx.core.player(from).equips.contains(&card);
                    if was_equip {
                        ctx.core.player_mut(from).equips.retain(|c| *c != card);
                        crate::kernel::revoke_equip_skills(ctx.core, ctx.catalog, from, card);
                        ctx.core.player_mut(to).equips.push(card);
                        crate::kernel::grant_equip_skills(ctx, to, card);
                    } else {
                        ctx.core.player_mut(from).judge.retain(|c| *c != card);
                        ctx.note_cards_lost(from, &[card]);
                        ctx.core.player_mut(to).judge.push(card);
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
