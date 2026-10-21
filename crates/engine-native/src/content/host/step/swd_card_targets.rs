use super::super::*;

impl HostMachine {
    pub(super) fn step_swd_card_targets(&mut self, ctx: &mut Ctx<'_>) -> Flow {
        match self.behavior {
            // 鬼眼：SelectCards 取目标手牌一张梅花 → 获得之
            HostBehavior::SwdGuiyan => match self.stage {
                HostStage::Act => {
                    let options: Vec<ChoiceOption> = ctx
                        .core
                        .alive_players()
                        .filter(|p| {
                            *p != self.owner
                                && ctx.core.player(*p).hand.iter().any(|c| {
                                    matches!(
                                        ctx.catalog.cards[*c as usize].suit,
                                        crate::catalog::Suit::Club
                                    )
                                })
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
                            prompt: "guiyan_target",
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
                    self.target = Some(t);
                    let clubs: Vec<CardId> = ctx
                        .core
                        .player(t)
                        .hand
                        .iter()
                        .copied()
                        .filter(|c| {
                            matches!(
                                ctx.catalog.cards[*c as usize].suit,
                                crate::catalog::Suit::Club
                            )
                        })
                        .collect();
                    if clubs.is_empty() {
                        return Flow::Done;
                    }
                    self.cards = clubs;
                    self.stage = HostStage::SelectCard;
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectCards,
                            prompt: "guiyan_take",
                            options: self.cards.iter().copied().map(ChoiceOption::Card).collect(),
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
                    let Some(t) = self.target else {
                        return Flow::Done;
                    };
                    if ctx.core.player(t).hand.contains(&picked) {
                        ctx.take_hand_card(t, picked);
                        ctx.core.player_mut(self.owner).hand.push(picked);
                        ctx.log(LogEvent::CardsGiven {
                            from: t,
                            to: self.owner,
                            cards: vec![picked],
                        });
                        // 复活面实例授予（上游 group guiyan2 随用随挂；
                        // def 名=cards.json 顶层技能名 swd_guiyan2）
                        if let Some(g2) = ctx.catalog.skill_by_name("swd_guiyan2") {
                            ctx.core.skills.entry((self.owner, g2)).or_insert(
                                crate::state::SkillInstance {
                                    owner: self.owner,
                                    enabled: true,
                                    once_used: false,
                                    expires_end_of_turn: false,
                                    rounds: 0,
                                },
                            );
                        }
                    }
                    self.cards.clear();
                    Flow::Done
                }
                _ => Flow::Done,
            },
            // 花间·剑仙面：SelectCard 弃一张 he → TargetChoice 杀合法目标 →
            // 虚拟杀
            HostBehavior::SwdHuajian => match self.stage {
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
                            prompt: "huajian_discard",
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
                            prompt: "huajian_target",
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
                        ctx.use_virtual_card(self.owner, CardKind::Sha, vec![t]);
                    }
                    Flow::Done
                }
                _ => Flow::Done,
            },
            // 枕流·临月面：SelectCard 红色手牌 → TargetChoice 其他未挂附
            // 存活 → off_court 挂附（上游 placeCard 武将牌上同口径；
            // 触发面 SwdZhenjiu2 在其准备阶段取回）
            HostBehavior::SwdZhenjiu => match self.stage {
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
                            prompt: "zhenjiu_pick",
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
                    self.card = Some(c);
                    let options: Vec<ChoiceOption> = ctx
                        .core
                        .alive_players()
                        .filter(|p| *p != self.owner && ctx.core.player(*p).off_court.is_empty())
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
                            prompt: "zhenjiu_target",
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
                    let Some(c) = self.card else {
                        return Flow::Done;
                    };
                    if ctx.core.player(t).alive
                        && ctx.core.player(t).off_court.is_empty()
                        && ctx.core.player(self.owner).hand.contains(&c)
                    {
                        ctx.take_hand_card(self.owner, c);
                        ctx.core.player_mut(t).off_court.push(c);
                        // 授予触发面实例（其准备阶段取回+回复后自清）
                        if let Some(def) = ctx.catalog.skill_by_name(SWD_ZHENJIU2_KEY) {
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
                    Flow::Done
                }
                _ => Flow::Done,
            },
            // 麻醉·临月面：SelectCard 黑色手牌 → TargetChoice → off_court
            // 挂附（触发面 SwdLmazui2 在持有者伤害前置返还+降 1）
            HostBehavior::SwdLmazui => match self.stage {
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
                            prompt: "lmazui_pick",
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
                            crate::catalog::Suit::Spade | crate::catalog::Suit::Club
                        )
                    {
                        return Flow::Done;
                    }
                    self.card = Some(c);
                    let options: Vec<ChoiceOption> = ctx
                        .core
                        .alive_players()
                        .filter(|p| *p != self.owner && ctx.core.player(*p).off_court.is_empty())
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
                            prompt: "lmazui_target",
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
                    let Some(c) = self.card else {
                        return Flow::Done;
                    };
                    if ctx.core.player(t).alive
                        && ctx.core.player(t).off_court.is_empty()
                        && ctx.core.player(self.owner).hand.contains(&c)
                    {
                        ctx.take_hand_card(self.owner, c);
                        ctx.core.player_mut(t).off_court.push(c);
                        // 授予触发面实例（其伤害前置返还+降 1 后自清）
                        if let Some(def) = ctx.catalog.skill_by_name(SWD_LMAZUI2_KEY) {
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
                    Flow::Done
                }
                _ => Flow::Done,
            },
            // 玄咒：SelectCard 置出一张普通锦囊（discard:false=置出而非
            // 弃——原生近似弃置登记）→ TargetChoice 任一角色（含自己，
            // 判定区无同名牌）→ 确定性 kind 规则置入判定区
            HostBehavior::SwdXuanzhou => match self.stage {
                HostStage::Act => {
                    let tricks: Vec<ChoiceOption> = ctx
                        .core
                        .player(self.owner)
                        .hand
                        .iter()
                        .copied()
                        .filter(|c| {
                            let k = ctx.catalog.card_kind(*c);
                            !k.is_basic_kind() && k.equip_slot().is_none() && k != CardKind::Jiguan
                        })
                        .map(ChoiceOption::Card)
                        .collect();
                    if tricks.is_empty() {
                        return Flow::Done;
                    }
                    self.stage = HostStage::SelectCard;
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectCards,
                            prompt: "xuanzhou_trick",
                            options: tricks,
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
                    if !ctx.core.player(self.owner).hand.contains(&picked) {
                        return Flow::Done;
                    }
                    self.card = Some(picked);
                    let options: Vec<ChoiceOption> = ctx
                        .core
                        .alive_players()
                        .filter(|p| {
                            // 判定区无同名延时（乐/兵互不冲突）
                            ctx.core.player(*p).judge.iter().all(|j| {
                                let jk = ctx.catalog.card_kind(*j);
                                jk != CardKind::Le && jk != CardKind::Bing
                            })
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
                            prompt: "xuanzhou_target",
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
                    let Some(c) = self.card else {
                        return Flow::Done;
                    };
                    if !ctx.core.player(t).alive {
                        return Flow::Done;
                    }
                    // 确定性 kind：目标手牌≤1→兵；余→乐（上游 AI 序近似，
                    // 玩家自选面登记差异）
                    let kind = if ctx.core.player(t).hand.len() <= 1 {
                        CardKind::Bing
                    } else {
                        CardKind::Le
                    };
                    ctx.take_hand_card(self.owner, c);
                    ctx.to_discard(c);
                    // 延时牌置入判定区：直接造该 kind 的判定实体
                    let judge_card = ctx
                        .core
                        .deck
                        .iter()
                        .copied()
                        .find(|x| ctx.catalog.card_kind(*x) == kind)
                        .or_else(|| {
                            let found = ctx
                                .core
                                .discard
                                .iter()
                                .copied()
                                .find(|x| ctx.catalog.card_kind(*x) == kind);
                            found
                        });
                    let Some(jc) = judge_card else {
                        return Flow::Done;
                    };
                    if ctx.core.deck.contains(&jc) {
                        ctx.core.deck.retain(|x| *x != jc);
                    } else {
                        ctx.core.discard.retain(|x| *x != jc);
                    }
                    ctx.core.player_mut(t).judge.push(jc);
                    ctx.emit(EventData::CardsMoved {
                        cards: vec![jc],
                        from: self.owner,
                        to: t,
                    });
                    self.card = None;
                    Flow::Done
                }
                _ => Flow::Done,
            },
            // 孵鸟：SelectCard 一张手牌 → TargetChoice 其他角色 → 交牌
            // + 摸 1（观看手牌=展示面登记差异）
            HostBehavior::SwdFuniao => match self.stage {
                HostStage::Act => {
                    if ctx.core.player(self.owner).hand.is_empty() {
                        return Flow::Done;
                    }
                    self.stage = HostStage::SelectCard;
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectCards,
                            prompt: "funiao_give",
                            options: ctx
                                .core
                                .player(self.owner)
                                .hand
                                .iter()
                                .copied()
                                .map(ChoiceOption::Card)
                                .collect(),
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
                    if !ctx.core.player(self.owner).hand.contains(&picked) {
                        return Flow::Done;
                    }
                    self.card = Some(picked);
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
                            prompt: "funiao_target",
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
                    let Some(c) = self.card else {
                        return Flow::Done;
                    };
                    if ctx.core.player(self.owner).hand.contains(&c) && ctx.core.player(t).alive {
                        ctx.take_hand_card(self.owner, c);
                        ctx.core.player_mut(t).hand.push(c);
                        ctx.log(LogEvent::CardsGiven {
                            from: self.owner,
                            to: t,
                            cards: vec![c],
                        });
                        ctx.draw_cards(self.owner, 1);
                    }
                    self.card = None;
                    Flow::Done
                }
                _ => Flow::Done,
            },
            // 剑击·少微面：SelectCard 弃装备牌 → TargetChoice 杀合法目标 →
            // 虚拟杀 + 摸 1（上游 useCard sha + draw 同口径）
            HostBehavior::SwdJianji => match self.stage {
                HostStage::Act => {
                    let options: Vec<ChoiceOption> = ctx
                        .core
                        .player(self.owner)
                        .hand
                        .iter()
                        .copied()
                        .chain(ctx.core.player(self.owner).equips.iter().copied())
                        .filter(|c| ctx.catalog.card_kind(*c).is_equip())
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
                            prompt: "jianji_pick",
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
                        .filter(|p| {
                            *p != self.owner
                                && ctx.core.computed_distance(ctx.catalog, self.owner, *p)
                                    <= ctx.core.attack_range(ctx.catalog, self.owner)
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
                            prompt: "jianji_target",
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
                    if ctx.core.player(t).alive && !ctx.core.deck.is_empty() {
                        ctx.use_virtual_card(self.owner, CardKind::Sha, vec![t]);
                        ctx.draw_cards(self.owner, 1);
                    }
                    Flow::Done
                }
                _ => Flow::Done,
            },
            // 命符·姜子牙面：SelectCard 梅花手牌 → TargetChoice 任一其他
            // 存活 → 弃梅花牌+实体发现鬼幽结置其判定区（上游 prepare throw
            // + addJudge 同口径；登记差异：实体发现替代 createCard；同名
            // 判定区重复由命令期校验承担）
            HostBehavior::SwdMingfu => match self.stage {
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
                                crate::catalog::Suit::Club
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
                            prompt: "mingfu_pick",
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
                            crate::catalog::Suit::Club
                        )
                    {
                        return Flow::Done;
                    }
                    ctx.discard_from_hand(self.owner, c);
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
                            prompt: "mingfu_target",
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
                    // 实体发现鬼幽结（牌堆优先，弃堆兜底；均无则不做）
                    let found = ctx
                        .core
                        .deck
                        .iter()
                        .position(|c| ctx.catalog.card_kind(*c) == CardKind::Guiyoujie)
                        .map(|i| ctx.core.deck.remove(i))
                        .or_else(|| {
                            ctx.core
                                .discard
                                .iter()
                                .position(|c| ctx.catalog.card_kind(*c) == CardKind::Guiyoujie)
                                .map(|i| ctx.core.discard.remove(i))
                        });
                    if let Some(card) = found {
                        ctx.core.player_mut(t).judge.push(card);
                    }
                    Flow::Done
                }
                _ => Flow::Done,
            },
            // 幻形（武将面批 87）：SelectCard 弃一张 he → TargetChoice
            // 男性其他存活角色 → 复制其（非 unique/未持有）技能实例并
            // 置 huanxing_granted（受伤后移除）
            HostBehavior::SwdHuanxing => match self.stage {
                HostStage::Act => {
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
                            prompt: "huanxing_discard",
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
                    let options: Vec<ChoiceOption> = ctx
                        .core
                        .alive_players()
                        .filter(|p| {
                            *p != self.owner
                                && ctx.core.effective_gender(*p)
                                    == crate::content::standard::Gender::Male
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
                            prompt: "huanxing_target",
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
                    // 复制目标将的技能名（登记差异：unique/已持有过滤按
                    // 目录名；复制面收敛为实例授予）
                    let names: Vec<&'static str> = {
                        let meta = ctx
                            .catalog
                            .general_meta(ctx.core.player(t).general_slots[0].unwrap_or(u16::MAX));
                        meta.map(|(_, _, skills, _, _)| skills).unwrap_or_default()
                    };
                    let mut granted = Vec::new();
                    for n in names {
                        if let Some(def) = ctx.catalog.skill_by_name(n) {
                            if !ctx.core.skills.contains_key(&(self.owner, def)) {
                                granted.push(def);
                            }
                        }
                    }
                    if granted.is_empty() {
                        return Flow::Done;
                    }
                    for def in granted {
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
                    // 在位旗标=内部状态技能实例（免存档字段批——
                    // 已启用技能随内容锁/快照自然往返）
                    if let Some(sdef) = ctx.catalog.skill_by_name(SWD_HUANXING_STATE_KEY) {
                        ctx.core.skills.insert(
                            (self.owner, sdef),
                            crate::state::SkillInstance {
                                owner: self.owner,
                                enabled: true,
                                once_used: false,
                                expires_end_of_turn: false,
                                rounds: 0,
                            },
                        );
                    }
                    Flow::Done
                }
                _ => Flow::Done,
            },
            // 愚钝·福面：SelectCards 两张锦囊手牌（消费段校验）→
            // TargetChoice 攻击距离内其他存活 → 首张作 declared-Sha 载荷
            //（use_as 声明）进入杀结算，次张弃置（丈八蛇矛机同型，
            // 不计出杀次数经 kernel 豁免臂）
            HostBehavior::SwdYudun => match self.stage {
                HostStage::Act => {
                    let options: Vec<ChoiceOption> = ctx
                        .core
                        .player(self.owner)
                        .hand
                        .iter()
                        .copied()
                        .filter(|c| {
                            let k = ctx.catalog.card_kind(*c);
                            !k.is_basic_kind()
                                && k.equip_slot().is_none()
                                && k.is_basic_trick_or_equip()
                        })
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
                            prompt: "yudun_pick_two",
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
                    if picks.len() != 2 {
                        return Flow::Done;
                    }
                    // 消费段校验：两张均为锦囊且在手
                    let all_tricks = picks.iter().all(|c| {
                        let k = ctx.catalog.card_kind(*c);
                        !k.is_basic_kind()
                            && k.equip_slot().is_none()
                            && k.is_basic_trick_or_equip()
                    });
                    if !all_tricks
                        || picks
                            .iter()
                            .any(|c| !ctx.core.player(self.owner).hand.contains(c))
                    {
                        return Flow::Done;
                    }
                    self.cards = picks;
                    let options: Vec<ChoiceOption> = ctx
                        .core
                        .alive_players()
                        .filter(|p| {
                            *p != self.owner
                                && ctx.core.computed_distance(ctx.catalog, self.owner, *p)
                                    <= ctx.core.attack_range(ctx.catalog, self.owner)
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
                            prompt: "yudun_sha_target",
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
                            let [c1, c2] = self.cards.as_slice() else {
                                return Flow::Done;
                            };
                            let (c1, c2) = (*c1, *c2);
                            // 次张弃为转化素材；首张作 declared-Sha 载荷
                            // 进入杀结算（丈八蛇矛同型）
                            ctx.discard_from_hand(self.owner, c2);
                            if ctx.core.player(self.owner).hand.contains(&c1) {
                                ctx.core.declared_kinds.insert(c1, CardKind::Sha);
                                ctx.emit(EventData::CardUsed(crate::machine::card_use_as(
                                    c1,
                                    self.owner,
                                    vec![t],
                                    Some(CardKind::Sha),
                                )));
                            }
                        }
                    }
                    Flow::Done
                }
                _ => Flow::Done,
            },
            // 运筹·姬亮面：SelectCard 弃一张手牌 → TargetChoice 有手牌
            // 目标 → 目标随机弃一张手牌：同色自己摸 1 否则目标摸 1
            HostBehavior::SwdYunchou => match self.stage {
                HostStage::Act => {
                    if ctx.core.player(self.owner).hand.is_empty() {
                        return Flow::Done;
                    }
                    self.stage = HostStage::SelectCard;
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectCards,
                            prompt: "yunchou_discard",
                            options: ctx
                                .core
                                .player(self.owner)
                                .hand
                                .iter()
                                .copied()
                                .map(ChoiceOption::Card)
                                .collect(),
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
                    if !ctx.core.player(self.owner).hand.contains(&picked) {
                        return Flow::Done;
                    }
                    ctx.discard_from_hand(self.owner, picked);
                    let anchor_color = ctx.catalog.cards[picked as usize].suit;
                    let is_red_anchor = matches!(
                        anchor_color,
                        crate::catalog::Suit::Heart | crate::catalog::Suit::Diamond
                    );
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
                    self.card = Some(u32::from(is_red_anchor));
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectPlayers,
                            prompt: "yunchou_target",
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
                    if !ctx.core.player(t).alive || ctx.core.player(t).hand.is_empty() {
                        return Flow::Done;
                    }
                    let pool: Vec<CardId> = ctx.core.player(t).hand.iter().copied().collect();
                    let j = ctx.core.rng.gen_range(pool.len() as u32) as usize;
                    let c = pool[j];
                    let same = {
                        let t_suit = ctx.catalog.cards[c as usize].suit;
                        let is_red_t = matches!(
                            t_suit,
                            crate::catalog::Suit::Heart | crate::catalog::Suit::Diamond
                        );
                        let is_red_a = self.card == Some(1);
                        is_red_t == is_red_a
                    };
                    ctx.discard_from_hand(t, c);
                    if same {
                        ctx.draw_cards(self.owner, 1);
                    } else {
                        ctx.draw_cards(t, 1);
                    }
                    self.card = None;
                    Flow::Done
                }
                _ => Flow::Done,
            },
            // 分舵（习嫣 jiefen）：三段——选交牌目标（手>自己）→ 目标选
            // 手牌交出 → 自己选 1 张手牌交给手牌最少的其他存活角色（登
            // 记差异：上游 chooseCardTarget 同时选牌与目标，原生确定性
            // 目标两段询问）。
            HostBehavior::SwdJiefen => match self.stage {
                HostStage::JiefenTarget => {
                    let my = ctx.core.player(self.owner).hand.len();
                    let options: Vec<ChoiceOption> = (0..ctx.core.players.len() as u8)
                        .filter(|&i| {
                            i != self.owner
                                && ctx.core.player(i).alive
                                && ctx.core.player(i).hand.len() > my
                        })
                        .map(ChoiceOption::Player)
                        .collect();
                    if options.is_empty() {
                        return Flow::Done;
                    }
                    self.stage = HostStage::JiefenTargetDiscard;
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectPlayers,
                            prompt: "swd_jiefen_target",
                            options,
                            min: 1,
                            max: 1,
                            allow_cancel: true,
                        },
                    }
                }
                HostStage::JiefenTargetDiscard => {
                    let Some(t) = ctx.answer().iter().find_map(|o| match o {
                        ChoiceOption::Player(p) => Some(*p),
                        _ => None,
                    }) else {
                        return Flow::Done;
                    };
                    self.target = Some(t);
                    self.stage = HostStage::JiefenOwnerDiscard;
                    let options: Vec<ChoiceOption> = ctx
                        .core
                        .player(t)
                        .hand
                        .iter()
                        .copied()
                        .map(ChoiceOption::Card)
                        .collect();
                    if options.is_empty() {
                        return Flow::Done;
                    }
                    Flow::WaitChoice {
                        observer: t,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectCards,
                            prompt: "swd_jiefen_give",
                            options,
                            min: 1,
                            max: 1,
                            allow_cancel: false,
                        },
                    }
                }
                HostStage::JiefenOwnerDiscard => {
                    let Some(t) = self.target else {
                        return Flow::Done;
                    };
                    let Some(c) = ctx.answer().iter().find_map(|o| match o {
                        ChoiceOption::Card(c) => Some(*c),
                        _ => None,
                    }) else {
                        return Flow::Done;
                    };
                    ctx.core.player_mut(t).hand.retain(|x| *x != c);
                    ctx.core.player_mut(self.owner).hand.push(c);
                    ctx.emit(EventData::CardsMoved {
                        cards: vec![c],
                        from: t,
                        to: self.owner,
                    });
                    ctx.log(LogEvent::CardsGiven {
                        from: t,
                        to: self.owner,
                        cards: vec![c],
                    });
                    // 自己交 1 张手牌给手牌最少的其他存活角色（确定性）
                    let dest = (0..ctx.core.players.len() as u8)
                        .filter(|&i| i != self.owner && ctx.core.player(i).alive)
                        .min_by_key(|&i| ctx.core.player(i).hand.len());
                    if let Some(d) = dest {
                        if let Some(c2) = ctx.core.player(self.owner).hand.first().copied() {
                            ctx.core.player_mut(self.owner).hand.retain(|x| *x != c2);
                            ctx.core.player_mut(d).hand.push(c2);
                            ctx.emit(EventData::CardsMoved {
                                cards: vec![c2],
                                from: self.owner,
                                to: d,
                            });
                            ctx.log(LogEvent::CardsGiven {
                                from: self.owner,
                                to: d,
                                cards: vec![c2],
                            });
                        }
                    }
                    Flow::Done
                }
                _ => Flow::Done,
            },
            // 林韵：两张牌当杀（手牌或装备；首张为载荷牌，declared Sha）
            HostBehavior::SwdLinyun => match self.stage {
                HostStage::Act => {
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
                    self.stage = HostStage::CardsAsk;
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectCards,
                            prompt: "linyun_pick_two",
                            options,
                            min: 2,
                            max: 2,
                            allow_cancel: false,
                        },
                    }
                }
                HostStage::CardsAsk => {
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
                    self.cards = picked;
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
                            prompt: "linyun_sha_target",
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
                    // 两张牌为转化素材（手牌/装备皆可）：次张弃置，首张为
                    // 载荷牌并声明为杀
                    for c in [*c2, *c1] {
                        if ctx.core.player(self.owner).hand.contains(&c) {
                            if c == *c1 {
                                continue;
                            }
                            ctx.discard_from_hand(self.owner, c);
                        } else if ctx.core.player(self.owner).equips.contains(&c) {
                            if c == *c1 {
                                continue;
                            }
                            ctx.core.player_mut(self.owner).equips.retain(|x| *x != c);
                            crate::kernel::revoke_equip_skills(
                                ctx.core,
                                ctx.catalog,
                                self.owner,
                                c,
                            );
                            ctx.to_discard(c);
                        } else {
                            return Flow::Done;
                        }
                    }
                    // 载荷牌须在手中（emit 后由杀结算弃置）
                    if ctx.core.player(self.owner).equips.contains(c1) {
                        ctx.core.player_mut(self.owner).equips.retain(|x| x != c1);
                        crate::kernel::revoke_equip_skills(ctx.core, ctx.catalog, self.owner, *c1);
                        ctx.core.player_mut(self.owner).hand.push(*c1);
                    }
                    if !ctx.core.player(self.owner).hand.contains(c1) {
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
                    // 置终态：子帧（杀结算）结束后本机再次恢复时直接收束，
                    // 不再读取已消费的应答
                    self.stage = HostStage::ShuchuiTargetLoop;
                    Flow::Yield
                }
                _ => Flow::Done,
            },
            // 魅惑：选目标 → 取其一张牌（装备牌则装备之）
            HostBehavior::SwdMeihuo => match self.stage {
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
                    if options.is_empty() {
                        return Flow::Done;
                    }
                    self.stage = HostStage::TargetChoice;
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectPlayers,
                            prompt: "meihuo_target",
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
                    self.target = Some(target);
                    let options: Vec<ChoiceOption> = ctx
                        .core
                        .player(target)
                        .hand
                        .iter()
                        .chain(ctx.core.player(target).equips.iter())
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
                            prompt: "meihuo_pick",
                            options,
                            min: 1,
                            max: 1,
                            allow_cancel: false,
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
                    let is_equip = ctx.catalog.card_kind(card).is_equip();
                    ctx.take_owned_card(target, card);
                    ctx.log(LogEvent::SkillTriggered {
                        skill: "魅惑",
                        owner: self.owner,
                    });
                    if is_equip {
                        // 装备牌：入手 → Equipped 帧（commit_equip 全托管）
                        ctx.core.player_mut(self.owner).hand.push(card);
                        ctx.emit(EventData::Equipped {
                            player: self.owner,
                            card,
                            replaced: None,
                        });
                    } else {
                        ctx.core.player_mut(self.owner).hand.push(card);
                    }
                    Flow::Done
                }
                _ => Flow::Done,
            },
            _ => unreachable!("behavior routed to the wrong step module"),
        }
    }
}
