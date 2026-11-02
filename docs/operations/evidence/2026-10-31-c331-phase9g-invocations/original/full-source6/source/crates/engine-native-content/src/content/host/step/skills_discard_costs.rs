//! 宿主技能：以弃牌或失去体力为代价的效果（修罗、灭道、轩辕、燎原、移花、贞烈）。

use super::super::*;

impl HostMachine {
    pub(super) fn step_skills_discard_costs(&mut self, ctx: &mut Ctx<'_>) -> Flow {
        match self.behavior {
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
                        let suit =
                            match ctx.res.frame(ctx.current_event_frame_id()).map(|f| &f.data) {
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
                    let frame = ctx.current_event_frame_id();
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
                    let user = match ctx.res.frame(ctx.current_event_frame_id()).map(|f| &f.data) {
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
            _ => unreachable!("behavior routed to the wrong step module"),
        }
    }
}
