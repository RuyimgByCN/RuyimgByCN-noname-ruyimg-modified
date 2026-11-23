//! 轩辕剑（swd）宿主技：付牌后按数量对多名目标造成伤害（奥剑、凝弦、活弹、占星）。

use super::super::*;

impl HostMachine {
    pub(super) fn step_swd_damage_multi(&mut self, ctx: &mut Ctx<'_>) -> Flow {
        match self.behavior {
            // 奥剑（swd/skill.js:4074）：SelectCard 暂存 1..min(X, 射程内人数, 手牌) →
            // TargetChoice 恰选 N 名射程内目标后提交弃牌并各受 1 伤；取消退回 once_used
            HostBehavior::SwdAojian => match self.stage {
                HostStage::Act => {
                    let x = (ctx.core.player(self.owner).max_hp as i32
                        - ctx.core.player(self.owner).hp)
                        .max(0) as usize;
                    let in_range = ctx
                        .core
                        .alive_players()
                        .filter(|p| {
                            *p != self.owner
                                && ctx.core.computed_distance(ctx.catalog, self.owner, *p) <= 1
                        })
                        .count();
                    let n = x.min(ctx.core.player(self.owner).hand.len()).min(in_range);
                    if n == 0 {
                        super::u05_7c_skills::refund_once_used(ctx, self.owner, &self.def_key);
                        return Flow::Done;
                    }
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
                            ChoiceOption::Card(c) if ctx.core.player(self.owner).hand.contains(c) => Some(*c),
                            _ => None,
                        })
                        .collect();
                    if picks.is_empty() {
                        super::u05_7c_skills::refund_once_used(ctx, self.owner, &self.def_key);
                        return Flow::Done;
                    }
                    let n = picks.len();
                    let options: Vec<ChoiceOption> = ctx
                        .core
                        .alive_players()
                        .filter(|p| {
                            *p != self.owner
                                && ctx.core.computed_distance(ctx.catalog, self.owner, *p) <= 1
                        })
                        .map(ChoiceOption::Player)
                        .collect();
                    if options.len() < n {
                        super::u05_7c_skills::refund_once_used(ctx, self.owner, &self.def_key);
                        return Flow::Done;
                    }
                    self.cards = picks;
                    self.stage = HostStage::TargetChoice;
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectPlayers,
                            prompt: "aojian_targets",
                            options,
                            min: n,
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
                    if targets.len() != n
                        || self
                            .cards
                            .iter()
                            .any(|c| !ctx.core.player(self.owner).hand.contains(c))
                    {
                        super::u05_7c_skills::refund_once_used(ctx, self.owner, &self.def_key);
                        self.cards.clear();
                        return Flow::Done;
                    }
                    let drop = std::mem::take(&mut self.cards);
                    for c in &drop {
                        ctx.discard_from_hand(self.owner, *c);
                    }
                    ctx.log(LogEvent::CardDiscarded {
                        player: self.owner,
                        cards: drop,
                    });
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
                    self.card = None;
                    Flow::Done
                }
                _ => Flow::Done,
            },
            // 凝弦：确认 → SelectCard 暂存黑色 he 牌 → SelectPlayers 选等量目标后弃牌并各受 1 伤
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
                                ctx.catalog.effective_card_suit(ctx.core, self.owner, *c),
                                crate::catalog::Suit::Spade | crate::catalog::Suit::Club
                            )
                        })
                        .map(ChoiceOption::Card)
                        .collect();
                    let others_len = ctx.core.alive_players().filter(|p| *p != self.owner).count();
                    let max = blacks.len().min(others_len);
                    if max == 0 {
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
                    if picks.is_empty() {
                        return Flow::Done;
                    }
                    let others: Vec<PlayerId> = ctx
                        .core
                        .alive_players()
                        .filter(|p| *p != self.owner)
                        .collect();
                    if others.len() < picks.len() {
                        return Flow::Done;
                    }
                    self.cards = picks;
                    self.stage = HostStage::TargetChoice;
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectPlayers,
                            prompt: "ningxian_targets",
                            options: others.into_iter().map(ChoiceOption::Player).collect(),
                            min: self.cards.len(),
                            max: self.cards.len(),
                            allow_cancel: true,
                        },
                    }
                }
                HostStage::TargetChoice => {
                    let picks = std::mem::take(&mut self.cards);
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
                    for c in &picks {
                        let _ = super::u05_7c_skills::discard_he_card(ctx, self.owner, *c);
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
                    Flow::Done
                }
                _ => Flow::Done,
            },
            // 活弹·舒莞儿面：SelectCard 暂存红牌（he 含装备）→ TargetChoice 选 1–2 目标 → 弃牌+失 1 体力并 Yield → DrawSettle 按序结算火伤
            HostBehavior::SwdHuodan => match self.stage {
                HostStage::Act => {
                    let options: Vec<ChoiceOption> = ctx
                        .core
                        .player(self.owner)
                        .hand
                        .iter()
                        .chain(ctx.core.player(self.owner).equips.iter())
                        .copied()
                        .filter(|c| {
                            matches!(
                                ctx.catalog.effective_card_suit(ctx.core, self.owner, *c),
                                crate::catalog::Suit::Heart | crate::catalog::Suit::Diamond
                            )
                        })
                        .map(ChoiceOption::Card)
                        .collect();
                    if options.is_empty() {
                        super::u05_7c_skills::refund_once_used(ctx, self.owner, &self.def_key);
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
                        super::u05_7c_skills::refund_once_used(ctx, self.owner, &self.def_key);
                        return Flow::Done;
                    };
                    let owned = ctx.core.player(self.owner).hand.contains(&c)
                        || ctx.core.player(self.owner).equips.contains(&c);
                    if !owned
                        || !matches!(
                            ctx.catalog.effective_card_suit(ctx.core, self.owner, c),
                            crate::catalog::Suit::Heart | crate::catalog::Suit::Diamond
                        )
                    {
                        super::u05_7c_skills::refund_once_used(ctx, self.owner, &self.def_key);
                        return Flow::Done;
                    }
                    self.card = Some(c);
                    let options: Vec<ChoiceOption> =
                        ctx.core.alive_players().map(ChoiceOption::Player).collect();
                    if options.is_empty() {
                        super::u05_7c_skills::refund_once_used(ctx, self.owner, &self.def_key);
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
                    let Some(c) = self.card.take() else {
                        super::u05_7c_skills::refund_once_used(ctx, self.owner, &self.def_key);
                        return Flow::Done;
                    };
                    if targets.is_empty()
                        || targets.len() > 2
                        || !super::u05_7c_skills::discard_he_card(ctx, self.owner, c)
                    {
                        super::u05_7c_skills::refund_once_used(ctx, self.owner, &self.def_key);
                        return Flow::Done;
                    }
                    self.targets = targets;
                    self.stage = HostStage::DrawSettle;
                    ctx.emit(EventData::LoseHp {
                        player: self.owner,
                        amount: 1,
                    });
                    Flow::Yield
                }
                HostStage::DrawSettle => {
                    let targets = std::mem::take(&mut self.targets);
                    if targets.len() == 1 {
                        if ctx.core.player(targets[0]).alive {
                            ctx.emit(EventData::Damage(crate::machine::damage_data_with(
                                Some(self.owner),
                                targets[0],
                                2,
                                crate::event::DamageNature::Fire,
                                None,
                            )));
                        }
                    } else {
                        // LIFO：逆序压栈以保证 targets[0] 先于 targets[1] 结算火伤
                        for t in targets.into_iter().rev() {
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
                    }
                    Flow::Done
                }
                _ => Flow::Done,
            },
            // 占星·肯迪面：SelectCards 弃任意张 he 牌 → 亮等量牌堆顶牌；取消退回 once_used
            HostBehavior::SwdZhanxing => match self.stage {
                HostStage::Act => {
                    let options = super::u05_7c_skills::he_card_options(ctx.core, self.owner);
                    if options.is_empty() {
                        super::u05_7c_skills::refund_once_used(ctx, self.owner, &self.def_key);
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
                            ChoiceOption::Card(c)
                                if ctx.core.player(self.owner).hand.contains(c)
                                    || ctx.core.player(self.owner).equips.contains(c) =>
                            {
                                Some(*c)
                            }
                            _ => None,
                        })
                        .collect();
                    if picks.is_empty() {
                        super::u05_7c_skills::refund_once_used(ctx, self.owner, &self.def_key);
                        return Flow::Done;
                    }
                    let n = picks.len();
                    for c in picks {
                        let _ = super::u05_7c_skills::discard_he_card(ctx, self.owner, c);
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
                        ctx.start_draw(self.owner, 2);
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
                    // 梅花：各敌人从 hand + equips 联合均匀随机弃 1
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
                            let h_len = ctx.core.player(e).hand.len();
                            let e_len = ctx.core.player(e).equips.len();
                            let total = h_len + e_len;
                            if total == 0 {
                                continue;
                            }
                            let j = ctx.core.rng.gen_range(total as u32) as usize;
                            let c = if j < h_len {
                                ctx.core.player(e).hand[j]
                            } else {
                                ctx.core.player(e).equips[j - h_len]
                            };
                            let _ = super::u05_7c_skills::discard_he_card(ctx, e, c);
                        }
                    }
                    // 黑桃：Yield 令方/红/梅先结算完毕，再于 DrawSettle 询问可取消目标雷伤
                    if suits.contains(&Suit::Spade) {
                        self.stage = HostStage::DrawSettle;
                        return Flow::Yield;
                    }
                    Flow::Done
                }
                HostStage::DrawSettle => {
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
                            prompt: "zhanxing_target",
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
