//! 轩辕剑（swd）宿主技：付牌后按数量对多名目标造成伤害（奥剑、凝弦、活弹、占星）。

use super::super::*;

impl HostMachine {
    pub(super) fn step_swd_damage_multi(&mut self, ctx: &mut Ctx<'_>) -> Flow {
        match self.behavior {
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
            _ => unreachable!("behavior routed to the wrong step module"),
        }
    }
}
