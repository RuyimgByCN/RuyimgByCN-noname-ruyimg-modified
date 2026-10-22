use super::super::*;

impl HostMachine {
    pub(super) fn step_skills_cards(&mut self, ctx: &mut Ctx<'_>) -> Flow {
        match self.behavior {
            // 起风：选一张未用过的可用锦囊牌名并使用（上游 chooseButton
            // + chooseUseTarget；原生以目录牌种码承载牌名选择）
            HostBehavior::HlssFengqi => match self.stage {
                HostStage::Act => {
                    let options: Vec<ChoiceOption> = hlss_fengqi_candidates(ctx, self.owner)
                        .into_iter()
                        .map(|k| ChoiceOption::Kind(crate::archive::kind_code(k)))
                        .collect();
                    if options.is_empty() {
                        return Flow::Done;
                    }
                    self.stage = HostStage::SelectCard;
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectCards,
                            prompt: "fengqi_card",
                            options,
                            min: 1,
                            max: 1,
                            allow_cancel: false,
                        },
                    }
                }
                HostStage::SelectCard => {
                    let Some(ChoiceOption::Kind(code)) = ctx.answer().first().copied() else {
                        return Flow::Done;
                    };
                    let Ok(kind) = crate::archive::kind_from_code(code) else {
                        return Flow::Done;
                    };
                    let (min, max) = hlss_fengqi_targets(kind);
                    if max == 0 {
                        // 无目标锦囊：直接视为使用（无中/五谷/桃园/南蛮/万箭）
                        let targets = hlss_fengqi_all_targets(ctx, self.owner, kind);
                        ctx.use_virtual_card(self.owner, kind, targets);
                        ctx.core.player_mut(self.owner).hlss_fengqi_used.push(code);
                        return Flow::Done;
                    }
                    let options: Vec<ChoiceOption> = ctx
                        .core
                        .alive_players()
                        .filter(|p| hlss_fengqi_target_ok(ctx, self.owner, *p, kind))
                        .map(ChoiceOption::Player)
                        .collect();
                    if (options.len() as u8) < min {
                        return Flow::Done;
                    }
                    self.cards.push(u32::from(code));
                    self.stage = HostStage::TargetChoice;
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectPlayers,
                            prompt: "fengqi_targets",
                            max: options.len().min(max as usize),
                            options,
                            min: min as usize,
                            allow_cancel: false,
                        },
                    }
                }
                HostStage::TargetChoice => {
                    let Some(code) = self.cards.first().copied() else {
                        return Flow::Done;
                    };
                    let Ok(kind) = crate::archive::kind_from_code(code as u16) else {
                        return Flow::Done;
                    };
                    let targets: Vec<PlayerId> = ctx
                        .answer()
                        .iter()
                        .filter_map(|o| match o {
                            ChoiceOption::Player(p) => Some(*p),
                            _ => None,
                        })
                        .collect();
                    if targets.is_empty() {
                        return Flow::Done;
                    }
                    ctx.use_virtual_card(self.owner, kind, targets);
                    ctx.core
                        .player_mut(self.owner)
                        .hlss_fengqi_used
                        .push(code as u16);
                    Flow::Done
                }
                _ => Flow::Done,
            },
            // 集控：选攻击距离内一名其他角色 → 虚拟雷杀（取消=不消耗次数）
            // 每步：弃一张牌 → 令目标本回合止息（+ 条件性距离视为 1）
            HostBehavior::NewMeibu => match self.stage {
                HostStage::Act => {
                    let Some(target) = self.target else {
                        return Flow::Done;
                    };
                    if !ctx.core.player(target).alive {
                        return Flow::Done;
                    }
                    let mut options: Vec<ChoiceOption> = ctx
                        .core
                        .player(self.owner)
                        .hand
                        .iter()
                        .chain(ctx.core.player(self.owner).equips.iter())
                        .copied()
                        .map(ChoiceOption::Card)
                        .collect();
                    if options.is_empty() {
                        return Flow::Done;
                    }
                    options.truncate(64);
                    self.stage = HostStage::SelectCard;
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectCards,
                            prompt: "new_meibu_discard",
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
                    if !ctx.core.player(target).alive {
                        return Flow::Done;
                    }
                    let is_sha = ctx.catalog.card_kind(card).is_sha_like();
                    let is_trick = {
                        let k = ctx.catalog.card_kind(card);
                        !k.is_basic_kind() && k.equip_slot().is_none()
                    };
                    let is_black = matches!(
                        ctx.catalog.cards[card as usize].suit,
                        crate::catalog::Suit::Spade | crate::catalog::Suit::Club
                    );
                    if ctx.core.player(self.owner).hand.contains(&card) {
                        ctx.discard_from_hand(self.owner, card);
                    } else if ctx.core.player(self.owner).equips.contains(&card) {
                        ctx.core
                            .player_mut(self.owner)
                            .equips
                            .retain(|c| *c != card);
                        crate::kernel::revoke_equip_skills(ctx.core, ctx.catalog, self.owner, card);
                        ctx.to_discard(card);
                    }
                    ctx.log(LogEvent::SkillTriggered {
                        skill: "每步",
                        owner: self.owner,
                    });
                    if let Some(def) = ctx.catalog.skill_by_name(L27_NEW_ZHIXI_KEY) {
                        ctx.core.skills.insert(
                            (target, def),
                            crate::state::SkillInstance {
                                owner: target,
                                enabled: true,
                                once_used: false,
                                expires_end_of_turn: true,
                                rounds: 0,
                            },
                        );
                        let hp = ctx.core.player(target).hp.max(1);
                        ctx.core.vm_aux.marks.insert((target, u32::from(def)), hp);
                    }
                    if !is_sha && !is_trick && !is_black {
                        if let Some(def) = ctx.catalog.skill_by_name(L27_NEW_MEIBU_RANGE_KEY) {
                            ctx.core.skills.insert(
                                (target, def),
                                crate::state::SkillInstance {
                                    owner: target,
                                    enabled: true,
                                    once_used: false,
                                    expires_end_of_turn: true,
                                    rounds: 0,
                                },
                            );
                            ctx.core
                                .vm_aux
                                .marks
                                .insert((target, u32::from(def)), i32::from(self.owner));
                        }
                    }
                    Flow::Done
                }
                _ => Flow::Done,
            },
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
                            // 获得牌：摸 1 → 选类型 → 牌堆取一张
                            ctx.draw_cards(self.owner, 1);
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
            // 蛊惑：弃一张手牌 → 声明基本/锦囊牌名 → 选目标 → 转化使用
            HostBehavior::Reguhuo => match self.stage {
                HostStage::Act => {
                    // 蛊惑：每回合一次（第 942 轮）——进入执行即记账，turn 结束清
                    // 蛊惑：每回合一次（第 942 轮）——进入执行即记账，turn 结束清
                    ctx.core
                        .vm_aux
                        .once
                        .insert((self.owner, self.def_key.clone(), 1));
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
                    self.stage = HostStage::SelectCard;
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectCards,
                            prompt: "reguhuo_card",
                            options,
                            min: 1,
                            max: 1,
                            allow_cancel: true,
                        },
                    }
                }
                HostStage::SelectCard => {
                    let Some(ChoiceOption::Card(card)) = ctx.answer().first().copied() else {
                        return Flow::Done;
                    };
                    if !ctx.core.player(self.owner).hand.contains(&card) {
                        return Flow::Done;
                    }
                    self.card = Some(card);
                    self.stage = HostStage::SelectNature;
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::PickTag,
                            prompt: "reguhuo_name",
                            options: reguhuo_kind_options(),
                            min: 1,
                            max: 1,
                            allow_cancel: true,
                        },
                    }
                }
                HostStage::SelectNature => {
                    let Some(ChoiceOption::Tag(idx)) = ctx.answer().first().copied() else {
                        return Flow::Done;
                    };
                    let Some(card) = self.card else {
                        return Flow::Done;
                    };
                    let Some(kind) = reguhuo_kind_at(idx) else {
                        return Flow::Done;
                    };
                    let (min, max, targets) = ctx.core.legal_targets(ctx.catalog, kind, self.owner);
                    self.card = Some(card);
                    // 存**下标**（非 CardKind 判别值）——TargetChoice 按同表映射
                    self.cards = vec![u32::from(idx)];
                    if min == 0 && max == 0 {
                        ctx.log(LogEvent::SkillTriggered {
                            skill: "蛊惑",
                            owner: self.owner,
                        });
                        ctx.emit(EventData::CardUsed(crate::machine::card_use_as(
                            card,
                            self.owner,
                            Vec::new(),
                            Some(kind),
                        )));
                        self.stage = HostStage::ShuchuiTargetLoop;
                        return Flow::Yield;
                    }
                    if targets.is_empty() {
                        return Flow::Done;
                    }
                    self.stage = HostStage::TargetChoice;
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectPlayers,
                            prompt: "reguhuo_targets",
                            options: targets.into_iter().map(ChoiceOption::Player).collect(),
                            min: min as usize,
                            max: max as usize,
                            allow_cancel: false,
                        },
                    }
                }
                HostStage::TargetChoice => {
                    let Some(card) = self.card else {
                        return Flow::Done;
                    };
                    let Some(kind) = self.cards.first().and_then(|k| reguhuo_kind_at(*k as u8))
                    else {
                        return Flow::Done;
                    };
                    let targets: Vec<PlayerId> = ctx
                        .answer()
                        .iter()
                        .filter_map(|o| match o {
                            ChoiceOption::Player(p) => Some(*p),
                            _ => None,
                        })
                        .collect();
                    if targets.is_empty() {
                        return Flow::Done;
                    }
                    ctx.log(LogEvent::SkillTriggered {
                        skill: "蛊惑",
                        owner: self.owner,
                    });
                    ctx.emit(EventData::CardUsed(crate::machine::card_use_as(
                        card,
                        self.owner,
                        targets,
                        Some(kind),
                    )));
                    self.stage = HostStage::ShuchuiTargetLoop;
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
                        owner: self.owner,
                    });
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
            // 修罗：选一张手牌 → 选一张同花色判定区牌 → 一并弃置
            HostBehavior::Xiuluo => match self.stage {
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
                    self.stage = HostStage::SelectCard;
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectCards,
                            prompt: "xiuluo_hand",
                            options,
                            min: 1,
                            max: 1,
                            allow_cancel: true,
                        },
                    }
                }
                HostStage::SelectCard => {
                    let Some(ChoiceOption::Card(hand)) = ctx.answer().first().copied() else {
                        return Flow::Done;
                    };
                    if !ctx.core.player(self.owner).hand.contains(&hand) {
                        return Flow::Done;
                    }
                    self.card = Some(hand);
                    let suit = ctx.catalog.cards[hand as usize].suit;
                    let options: Vec<ChoiceOption> = ctx
                        .core
                        .player(self.owner)
                        .judge
                        .iter()
                        .copied()
                        .filter(|c| ctx.catalog.cards[*c as usize].suit == suit)
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
                            prompt: "xiuluo_judge",
                            options,
                            min: 1,
                            max: 1,
                            allow_cancel: true,
                        },
                    }
                }
                HostStage::TargetChoice => {
                    let Some(ChoiceOption::Card(judge)) = ctx.answer().first().copied() else {
                        return Flow::Done;
                    };
                    let Some(hand) = self.card else {
                        return Flow::Done;
                    };
                    if !ctx.core.player(self.owner).judge.contains(&judge) {
                        return Flow::Done;
                    }
                    ctx.log(LogEvent::SkillTriggered {
                        skill: "修罗",
                        owner: self.owner,
                    });
                    ctx.discard_from_hand(self.owner, hand);
                    ctx.core
                        .player_mut(self.owner)
                        .judge
                        .retain(|c| *c != judge);
                    // 判定区失去牌（origins 由内核按实际所在区推断）
                    ctx.note_cards_lost(self.owner, &[judge]);
                    ctx.to_discard(judge);
                    Flow::Done
                }
                _ => Flow::Done,
            },
            // 灭道·弃：强制弃「已损失体力」张（手/装备）
            HostBehavior::Miedao => match self.stage {
                HostStage::Act => {
                    let gap = (ctx.core.player(self.owner).max_hp as i32
                        - ctx.core.player(self.owner).hp)
                        .max(0) as usize;
                    let cards: Vec<CardId> = ctx
                        .core
                        .player(self.owner)
                        .hand
                        .iter()
                        .chain(ctx.core.player(self.owner).equips.iter())
                        .copied()
                        .collect();
                    let need = cards.len().min(gap);
                    if need == 0 {
                        return Flow::Done;
                    }
                    self.cards = cards.iter().map(|c| u32::from(*c)).collect();
                    self.stage = HostStage::SelectCard;
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectCards,
                            prompt: "miedao_discard",
                            options: cards.into_iter().map(ChoiceOption::Card).collect(),
                            min: need,
                            max: need,
                            allow_cancel: false,
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
                    if picked.is_empty() {
                        return Flow::Done;
                    }
                    ctx.log(LogEvent::SkillTriggered {
                        skill: "灭道",
                        owner: self.owner,
                    });
                    for c in picked {
                        if ctx.core.player(self.owner).hand.contains(&c) {
                            ctx.discard_from_hand(self.owner, c);
                        } else if ctx.core.player(self.owner).equips.contains(&c) {
                            ctx.core.player_mut(self.owner).equips.retain(|x| *x != c);
                            crate::kernel::revoke_equip_skills(
                                ctx.core,
                                ctx.catalog,
                                self.owner,
                                c,
                            );
                            ctx.to_discard(c);
                        }
                    }
                    Flow::Done
                }
                _ => Flow::Done,
            },
            // 轩辕：弃一张黑桃牌 → 从弃牌堆取【轩辕剑】装备（限定）
            HostBehavior::Xuanyuan => match self.stage {
                HostStage::Act => {
                    let options: Vec<ChoiceOption> = ctx
                        .core
                        .player(self.owner)
                        .hand
                        .iter()
                        .chain(ctx.core.player(self.owner).equips.iter())
                        .copied()
                        .filter(|c| {
                            ctx.catalog.cards[*c as usize].suit == crate::catalog::Suit::Spade
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
                            prompt: "xuanyuan_discard",
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
                    if ctx.core.player(self.owner).hand.contains(&c) {
                        ctx.discard_from_hand(self.owner, c);
                    } else if ctx.core.player(self.owner).equips.contains(&c) {
                        ctx.core.player_mut(self.owner).equips.retain(|x| *x != c);
                        crate::kernel::revoke_equip_skills(ctx.core, ctx.catalog, self.owner, c);
                        ctx.to_discard(c);
                    } else {
                        return Flow::Done;
                    }
                    // 弃牌堆取【轩辕剑】入手后经装备帧入槽
                    let sword = ctx.core.discard.iter().copied().find(|c| {
                        ctx.catalog.cards[*c as usize].kind
                            == crate::catalog::CardKind::Xuanyuanjian
                    });
                    if let Some(sword) = sword {
                        ctx.core.discard.retain(|x| *x != sword);
                        ctx.core.player_mut(self.owner).hand.push(sword);
                        ctx.log(LogEvent::SkillTriggered {
                            skill: "轩辕",
                            owner: self.owner,
                        });
                        ctx.emit(EventData::Equipped {
                            player: self.owner,
                            card: sword,
                            replaced: None,
                        });
                        self.stage = HostStage::ShuchuiTargetLoop;
                        return Flow::Yield;
                    }
                    Flow::Done
                }
                _ => Flow::Done,
            },
            // 燎原：循环弃同花色牌（首张必问；弃后余 >1 张才续问）→ 记额外闪数
            HostBehavior::Liaoyuan => match self.stage {
                HostStage::Act => {
                    // 花色仅首次捕获（弃牌会发射帧，之后当前帧不再是 CardUsed）
                    if self.card.is_none() {
                        let suit = match ctx.res.frame(ctx.current_frame_id()).map(|f| &f.data) {
                            Some(EventData::CardUsed(u)) => {
                                ctx.catalog.cards[u.card as usize].suit as u8
                            }
                            _ => return Flow::Done,
                        };
                        self.card = Some(u32::from(suit));
                    }
                    let suit = self.card.unwrap_or(0) as u8;
                    let cards: Vec<CardId> = ctx
                        .core
                        .player(self.owner)
                        .hand
                        .iter()
                        .chain(ctx.core.player(self.owner).equips.iter())
                        .copied()
                        .filter(|c| mbcm_suit_of(ctx, *c) == Some(suit))
                        .collect();
                    // 上游循环条件：首张必问；弃后余 >1 张才续问
                    if cards.is_empty() || (!self.cards.is_empty() && cards.len() <= 1) {
                        let count = self.cards.len() as u8;
                        ctx.core.turn.liaoyuan_count = count;
                        if count > 0 {
                            ctx.log(LogEvent::SkillTriggered {
                                skill: "燎原",
                                owner: self.owner,
                            });
                        }
                        return Flow::Done;
                    }
                    self.stage = HostStage::SelectCard;
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectCards,
                            prompt: "liaoyuan_discard",
                            options: cards.into_iter().map(ChoiceOption::Card).collect(),
                            min: 1,
                            max: 1,
                            allow_cancel: true,
                        },
                    }
                }
                HostStage::SelectCard => {
                    let Some(ChoiceOption::Card(c)) = ctx.answer().first().copied() else {
                        // 取消：结束循环，记数
                        let count = self.cards.len() as u8;
                        ctx.core.turn.liaoyuan_count = count;
                        if count > 0 {
                            ctx.log(LogEvent::SkillTriggered {
                                skill: "燎原",
                                owner: self.owner,
                            });
                        }
                        return Flow::Done;
                    };
                    if ctx.core.player(self.owner).hand.contains(&c) {
                        ctx.discard_from_hand(self.owner, c);
                    } else if ctx.core.player(self.owner).equips.contains(&c) {
                        ctx.core.player_mut(self.owner).equips.retain(|x| *x != c);
                        crate::kernel::revoke_equip_skills(ctx.core, ctx.catalog, self.owner, c);
                        ctx.to_discard(c);
                    } else {
                        return Flow::Done;
                    }
                    self.cards.push(u32::from(c));
                    self.stage = HostStage::Act;
                    Flow::Yield
                }
                _ => Flow::Done,
            },
            // 移花：弃 2 张手牌 → 帧「使用者/目标」对调（反弹）
            HostBehavior::Yihua => match self.stage {
                HostStage::Act => {
                    let frame = ctx.current_frame_id();
                    let user = match ctx.res.frame(frame).map(|f| &f.data) {
                        Some(EventData::CardUsed(u)) => u.player,
                        _ => return Flow::Done,
                    };
                    self.target = Some(user);
                    self.card = Some(frame as CardId);
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
                    self.stage = HostStage::SelectCard;
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectCards,
                            prompt: "yihua_discard",
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
                        return Flow::Done;
                    }
                    let Some(user) = self.target else {
                        return Flow::Done;
                    };
                    let frame = self.card.unwrap_or(0) as crate::ids::EventId;
                    for c in picked {
                        if ctx.core.player(self.owner).hand.contains(&c) {
                            ctx.discard_from_hand(self.owner, c);
                        } else {
                            return Flow::Done;
                        }
                    }
                    ctx.log(LogEvent::SkillTriggered {
                        skill: "移花",
                        owner: self.owner,
                    });
                    // 帧改写：使用者=自己、目标=原使用者（反弹）
                    if let Some(f) = ctx.res.frame_mut(frame) {
                        if let EventData::CardUsed(u) = &mut f.data {
                            u.player = self.owner;
                            u.targets = vec![user];
                        }
                    }
                    Flow::Done
                }
                _ => Flow::Done,
            },
            // 贞烈：失去 1 体力 → 移出目标 → 弃置使用者一张牌（手/装备）
            HostBehavior::Zhenlie => match self.stage {
                HostStage::Act => {
                    let user = match ctx.res.frame(ctx.current_frame_id()).map(|f| &f.data) {
                        Some(EventData::CardUsed(u)) => u.player,
                        _ => return Flow::Done,
                    };
                    ctx.log(LogEvent::SkillTriggered {
                        skill: "贞烈",
                        owner: self.owner,
                    });
                    // ① 移出目标（结算前写回帧 targets——Intrinsic 阶段读取；
                    // 必须先于任何帧发射，否则当前帧不再是 CardUsed）
                    ctx.current_card_use_mut()
                        .targets
                        .retain(|t| *t != self.owner);
                    // ② 失去 1 点体力
                    ctx.emit(EventData::LoseHp {
                        player: self.owner,
                        amount: 1,
                    });
                    // ③ 弃置使用者一张牌（手/装备；强制）
                    let cards: Vec<CardId> = ctx
                        .core
                        .player(user)
                        .hand
                        .iter()
                        .chain(ctx.core.player(user).equips.iter())
                        .copied()
                        .collect();
                    if cards.is_empty() {
                        return Flow::Done;
                    }
                    self.target = Some(user);
                    self.cards = cards.iter().map(|c| u32::from(*c)).collect();
                    self.stage = HostStage::SelectCard;
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectCards,
                            prompt: "zhenlie_discard",
                            options: cards.into_iter().map(ChoiceOption::Card).collect(),
                            min: 1,
                            max: 1,
                            allow_cancel: false,
                        },
                    }
                }
                HostStage::SelectCard => {
                    let Some(user) = self.target else {
                        return Flow::Done;
                    };
                    let Some(ChoiceOption::Card(c)) = ctx.answer().first().copied() else {
                        return Flow::Done;
                    };
                    if ctx.core.player(user).hand.contains(&c) {
                        ctx.discard_from_hand(user, c);
                    } else if ctx.core.player(user).equips.contains(&c) {
                        ctx.core.player_mut(user).equips.retain(|x| *x != c);
                        crate::kernel::revoke_equip_skills(ctx.core, ctx.catalog, user, c);
                        ctx.to_discard(c);
                    }
                    Flow::Done
                }
                _ => Flow::Done,
            },
            // 极略：看牌堆顶 3 张 → 选一张基本/锦囊牌 → 选目标 → 使用
            HostBehavior::Jilve => match self.stage {
                HostStage::Act => {
                    let top: Vec<CardId> = ctx.core.deck.iter().rev().take(3).copied().collect();
                    let options: Vec<ChoiceOption> = top
                        .into_iter()
                        .filter(|c| {
                            let kind = ctx.catalog.card_kind(*c);
                            if kind.is_basic_kind() || kind.equip_slot().is_none() {
                                // 基本牌或锦囊（非装备）且目标规则可满足
                                let (min, _, targets) =
                                    ctx.core.legal_targets(ctx.catalog, kind, self.owner);
                                min == 0 || !targets.is_empty()
                            } else {
                                false
                            }
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
                            prompt: "jilve_card",
                            options,
                            min: 1,
                            max: 1,
                            allow_cancel: true,
                        },
                    }
                }
                HostStage::SelectCard => {
                    let Some(ChoiceOption::Card(card)) = ctx.answer().first().copied() else {
                        return Flow::Done;
                    };
                    if !ctx.core.deck.contains(&card) {
                        return Flow::Done;
                    }
                    let kind = ctx.catalog.card_kind(card);
                    let (min, max, targets) = ctx.core.legal_targets(ctx.catalog, kind, self.owner);
                    self.card = Some(card);
                    if min == 0 && max == 0 {
                        // 无目标牌：直接使用（牌堆 → 手牌 → CardUsed）
                        ctx.core.deck.retain(|c| *c != card);
                        ctx.core.player_mut(self.owner).hand.push(card);
                        ctx.core.player_mut(self.owner).jilve_used += 1;
                        ctx.emit(EventData::CardUsed(crate::machine::card_use(
                            card,
                            self.owner,
                            Vec::new(),
                        )));
                        self.stage = HostStage::ShuchuiTargetLoop;
                        return Flow::Yield;
                    }
                    if targets.is_empty() {
                        return Flow::Done;
                    }
                    self.stage = HostStage::TargetChoice;
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectPlayers,
                            prompt: "jilve_targets",
                            options: targets.into_iter().map(ChoiceOption::Player).collect(),
                            min: min as usize,
                            max: max as usize,
                            allow_cancel: false,
                        },
                    }
                }
                HostStage::TargetChoice => {
                    let Some(card) = self.card else {
                        return Flow::Done;
                    };
                    let targets: Vec<PlayerId> = ctx
                        .answer()
                        .iter()
                        .filter_map(|o| match o {
                            ChoiceOption::Player(p) => Some(*p),
                            _ => None,
                        })
                        .collect();
                    if targets.is_empty() || !ctx.core.deck.contains(&card) {
                        return Flow::Done;
                    }
                    ctx.core.deck.retain(|c| *c != card);
                    ctx.core.player_mut(self.owner).hand.push(card);
                    ctx.core.player_mut(self.owner).jilve_used += 1;
                    ctx.log(LogEvent::SkillTriggered {
                        skill: "极略",
                        owner: self.owner,
                    });
                    ctx.emit(EventData::CardUsed(crate::machine::card_use(
                        card, self.owner, targets,
                    )));
                    self.stage = HostStage::ShuchuiTargetLoop;
                    Flow::Yield
                }
                HostStage::ShuchuiTargetLoop => Flow::Done,
                _ => Flow::Done,
            },
            // 此桥邪：失去装备后逐次使用机关牌（自目标/单目标/AOE）
            HostBehavior::Cyqiaoxie => match self.stage {
                HostStage::Act => {
                    let lost_equips = match ctx.res.frame(ctx.current_frame_id()).map(|f| &f.data) {
                        Some(EventData::CardsLost { origins, .. }) => origins
                            .iter()
                            .filter(|o| **o == crate::event::LostOrigin::Equip)
                            .count(),
                        _ => 0,
                    };
                    if lost_equips == 0 {
                        return Flow::Done;
                    }
                    self.cards = (0..lost_equips).map(|_| 0u32).collect();
                    self.stage = HostStage::ShuchuiTargetLoop;
                    Flow::Yield
                }
                HostStage::ShuchuiTargetLoop => {
                    if self.cards.is_empty() {
                        return Flow::Done;
                    }
                    let options = cyqiaoxie_candidates(ctx, self.owner);
                    if options.is_empty() {
                        return Flow::Done;
                    }
                    self.cards.pop();
                    self.stage = HostStage::SelectCard;
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectCards,
                            prompt: "cyqiaoxie_card",
                            options,
                            min: 1,
                            max: 1,
                            allow_cancel: true,
                        },
                    }
                }
                HostStage::SelectCard => {
                    let Some(ChoiceOption::Card(card)) = ctx.answer().first().copied() else {
                        return Flow::Done;
                    };
                    self.card = Some(card);
                    let name = ctx.catalog.cards[card as usize].name;
                    match cyqiaoxie_rule(name) {
                        Some(CyqiaoxieRule::AllOthers) => {
                            let targets: Vec<PlayerId> = ctx
                                .core
                                .seat_order_from(self.owner)
                                .into_iter()
                                .filter(|p| *p != self.owner && ctx.core.player(*p).alive)
                                .collect();
                            cyqiaoxie_use(ctx, self.owner, card, targets);
                            self.stage = HostStage::ShuchuiTargetLoop;
                            Flow::Yield
                        }
                        Some(CyqiaoxieRule::OneOther) => {
                            let options: Vec<ChoiceOption> = ctx
                                .core
                                .alive_players()
                                .filter(|p| *p != self.owner && cyqiaoxie_target_ok(ctx, *p, name))
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
                                    prompt: "cyqiaoxie_target",
                                    options,
                                    min: 1,
                                    max: 1,
                                    allow_cancel: false,
                                },
                            }
                        }
                        Some(CyqiaoxieRule::SelfTarget) => {
                            cyqiaoxie_use(ctx, self.owner, card, vec![self.owner]);
                            self.stage = HostStage::ShuchuiTargetLoop;
                            Flow::Yield
                        }
                        None => Flow::Done,
                    }
                }
                HostStage::TargetChoice => {
                    let Some(ChoiceOption::Player(target)) = ctx.answer().first().copied() else {
                        return Flow::Done;
                    };
                    let Some(card) = self.card else {
                        return Flow::Done;
                    };
                    cyqiaoxie_use(ctx, self.owner, card, vec![target]);
                    self.stage = HostStage::ShuchuiTargetLoop;
                    Flow::Yield
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
                        owner: self.owner,
                    });
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
                        ctx.draw_cards(target, 2);
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
