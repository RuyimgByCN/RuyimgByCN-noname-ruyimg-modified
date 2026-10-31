//! 轩辕剑（swd）宿主技：弃牌换取持续效果或标记（横刀、化金、仙匠、星点、待行、陷阱、入梦）。

use super::super::*;

impl HostMachine {
    pub(super) fn step_swd_cards(&mut self, ctx: &mut Ctx<'_>) -> Flow {
        match self.behavior {
            // 横刀·慧彦面：当前回合角色二选一窗（弃同花色 he 牌或被获得
            // 一张）——上游 chooseToDiscard 可取消=被获得面
            HostBehavior::SwdHwendao => match self.stage {
                HostStage::Act => {
                    let cur = ctx.core.turn.current;
                    self.target = Some(cur);
                    let suit = match ctx.current_card_use() {
                        Ok(u) => ctx.catalog.cards[u.card as usize].suit,
                        Err(_) => match ctx.res.frame(ctx.current_frame_id()).map(|f| &f.data) {
                            Some(crate::event::EventData::CardResponded { card, .. }) => {
                                ctx.catalog.cards[*card as usize].suit
                            }
                            _ => return Flow::Done,
                        },
                    };
                    self.card = None;
                    // 花色锚存 cards[0]（哨兵实体不可得——以 suit 编码入
                    // cards 首元素不可行，改存 target 上下文于 target 字段）
                    let options: Vec<ChoiceOption> = ctx
                        .core
                        .player(cur)
                        .hand
                        .iter()
                        .chain(ctx.core.player(cur).equips.iter())
                        .copied()
                        .filter(|c| ctx.catalog.cards[*c as usize].suit == suit)
                        .map(ChoiceOption::Card)
                        .collect();
                    if options.is_empty() {
                        // 无同花色：直接被获得面
                        return self.hwendao_gain_random(ctx, cur);
                    }
                    self.stage = HostStage::SelectCard;
                    Flow::WaitChoice {
                        observer: cur,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectCards,
                            prompt: "hwendao_discard_suit",
                            options,
                            min: 0,
                            max: 1,
                            allow_cancel: true,
                        },
                    }
                }
                HostStage::SelectCard => {
                    let picked = ctx.answer().iter().find_map(|o| match o {
                        ChoiceOption::Card(c) => Some(*c),
                        _ => None,
                    });
                    let Some(t) = self.target else {
                        return Flow::Done;
                    };
                    if let Some(c) = picked.filter(|c| {
                        ctx.core.player(t).hand.contains(c) || ctx.core.player(t).equips.contains(c)
                    }) {
                        let in_hand = ctx.core.player(t).hand.contains(&c);
                        if in_hand {
                            ctx.take_hand_card(t, c);
                        } else {
                            ctx.core.player_mut(t).equips.retain(|x| *x != c);
                            if ctx.catalog.card_kind(c).is_equip() {
                                crate::kernel::revoke_equip_skills(ctx.core, ctx.catalog, t, c);
                            }
                        }
                        ctx.to_discard(c);
                        ctx.log(LogEvent::CardDiscarded {
                            player: t,
                            cards: vec![c],
                        });
                        Flow::Done
                    } else {
                        // 取消=被获得面
                        self.hwendao_gain_random(ctx, t)
                    }
                }
                _ => Flow::Done,
            },
            // 化金：SelectCard 弃一张 he → 授予 swd_huajin2
            HostBehavior::SwdHuajin => match self.stage {
                HostStage::Act => {
                    let p = ctx.core.player(self.owner);
                    let options: Vec<ChoiceOption> = p
                        .hand
                        .iter()
                        .chain(p.equips.iter())
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
                            prompt: "huajin_discard",
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
                    let p = ctx.core.player(self.owner);
                    if !p.hand.contains(&c) && !p.equips.contains(&c) {
                        return Flow::Done;
                    }
                    ctx.take_owned_card(self.owner, c);
                    ctx.to_discard(c);
                    ctx.log(LogEvent::CardDiscarded {
                        player: self.owner,
                        cards: vec![c],
                    });
                    if let Some(def) = ctx.catalog.skill_by_name(SWD_HUAJIN2_KEY) {
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
                    Flow::Done
                }
                _ => Flow::Done,
            },
            // 仙匠·车芸面：SelectCard 目标装备区含未持有种（消费段校验）
            // → 实体发现同牌种牌入手+Equipped 帧全托管装备
            //（take_hand_card+同栏替换+grant_equip_skills——破茧同款）
            HostBehavior::SwdCyxianjiang => match self.stage {
                HostStage::Act => {
                    let target = ctx
                        .res
                        .frame(ctx.current_frame_id())
                        .map(|f| &f.data)
                        .and_then(|data| match data {
                            crate::event::EventData::CardUsed(u) => u.targets.first().copied(),
                            _ => None,
                        });
                    let Some(t) = target else {
                        return Flow::Done;
                    };
                    self.target = Some(t);
                    // 每目标一次（上游 `storage.cyxianjiang`）：整局 once 记账
                    ctx.core
                        .vm_aux
                        .once
                        .insert((t, format!("cyxianjiang:{}", self.owner), 3));
                    let options: Vec<ChoiceOption> = ctx
                        .core
                        .player(t)
                        .equips
                        .iter()
                        .copied()
                        .filter(|c| {
                            // 同过滤臂：按**牌名**比较（上游 `card.name`），不是牌种
                            let name = ctx.catalog.cards[*c as usize].name;
                            !ctx.core
                                .player(self.owner)
                                .hand
                                .iter()
                                .chain(ctx.core.player(self.owner).equips.iter())
                                .any(|o| ctx.catalog.cards[*o as usize].name == name)
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
                            prompt: "cyxianjiang_pick",
                            options,
                            min: 1,
                            max: 1,
                            allow_cancel: true,
                        },
                    }
                }
                HostStage::SelectCard => {
                    let Some(pick) = ctx.answer().iter().find_map(|o| match o {
                        ChoiceOption::Card(c) => Some(*c),
                        _ => None,
                    }) else {
                        return Flow::Done;
                    };
                    let kind = ctx.catalog.card_kind(pick);
                    // 实体发现（牌堆优先弃堆；无则不做）
                    let found = ctx
                        .core
                        .deck
                        .iter()
                        .position(|c| ctx.catalog.card_kind(*c) == kind)
                        .map(|i| ctx.core.deck.remove(i))
                        .or_else(|| {
                            ctx.core
                                .discard
                                .iter()
                                .position(|c| ctx.catalog.card_kind(*c) == kind)
                                .map(|i| ctx.core.discard.remove(i))
                        });
                    if let Some(card) = found {
                        ctx.core.player_mut(self.owner).hand.push(card);
                        ctx.emit(EventData::Equipped {
                            player: self.owner,
                            card,
                            replaced: None,
                        });
                    }
                    Flow::Done
                }
                _ => Flow::Done,
            },
            // 星点：SelectCard 弃 1 张手牌 → Act 确定性两名其他存活角色
            // 各弃手/装备区首张（登记差异：上游随机 2 名敌人各随机弃 1）
            HostBehavior::SwdXingdian => match self.stage {
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
                            prompt: "xingdian_card",
                            options,
                            min: 0,
                            max: 1,
                            allow_cancel: true,
                        },
                    }
                }
                HostStage::Act => {
                    // 收牌应答（SelectCard 直入 Act，每臂一答铁律）
                    let picked: Option<CardId> = ctx.answer().iter().find_map(|o| match o {
                        ChoiceOption::Card(c) => Some(*c),
                        _ => None,
                    });
                    let Some(discarded) = picked else {
                        return Flow::Done;
                    };
                    if ctx.core.player(self.owner).hand.contains(&discarded) {
                        ctx.discard_from_hand(self.owner, discarded);
                        ctx.log(LogEvent::CardDiscarded {
                            player: self.owner,
                            cards: vec![discarded],
                        });
                    }
                    let mut victims: Vec<PlayerId> = (0..ctx.core.players.len())
                        .filter(|&i| {
                            i as PlayerId != self.owner
                                && ctx.core.player(i as PlayerId).alive
                                && (!ctx.core.player(i as PlayerId).hand.is_empty()
                                    || !ctx.core.player(i as PlayerId).equips.is_empty())
                        })
                        .map(|i| i as PlayerId)
                        .collect();
                    victims.truncate(2);
                    for v in victims {
                        let Some(c) = ctx
                            .core
                            .player(v)
                            .hand
                            .iter()
                            .chain(ctx.core.player(v).equips.iter())
                            .copied()
                            .next()
                        else {
                            continue;
                        };
                        if ctx.core.player(v).hand.contains(&c) {
                            ctx.discard_from_hand(v, c);
                        } else {
                            ctx.core.player_mut(v).equips.retain(|x| *x != c);
                            crate::kernel::revoke_equip_skills(ctx.core, ctx.catalog, v, c);
                            ctx.to_discard(c);
                        }
                        ctx.log(LogEvent::CardDiscarded {
                            player: v,
                            cards: vec![c],
                        });
                    }
                    Flow::Done
                }
                _ => Flow::Done,
            },
            // 待行：结束阶段弃 1..张 he 换取等量护甲
            HostBehavior::SwdDaixing => match self.stage {
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
                    if options.is_empty() {
                        return Flow::Done;
                    }
                    self.stage = HostStage::SelectCard;
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectCards,
                            prompt: "daixing_discard",
                            options,
                            min: 1,
                            max: 4,
                            allow_cancel: true,
                        },
                    }
                }
                HostStage::SelectCard => {
                    let picked: Vec<crate::ids::CardId> = ctx
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
                    let n = picked.len() as u8;
                    for c in picked {
                        if ctx.core.player(self.owner).hand.contains(&c) {
                            ctx.discard_from_hand(self.owner, c);
                        } else {
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
                            ctx.log(LogEvent::CardDiscarded {
                                player: self.owner,
                                cards: vec![c],
                            });
                        }
                    }
                    ctx.change_hujia(self.owner, n as i32);
                    ctx.core.player_mut(self.owner).daixing_count = n;
                    Flow::Done
                }
                _ => Flow::Done,
            },
            // 陷阱：出牌阶段置出一张花色未在陷阱表中的手牌为陷阱
            HostBehavior::SwdSxianjing => match self.stage {
                HostStage::Act => {
                    let traps: Vec<crate::catalog::Suit> = ctx
                        .core
                        .player(self.owner)
                        .sxianjing_traps
                        .iter()
                        .map(|c| ctx.catalog.cards[*c as usize].suit)
                        .collect();
                    let options: Vec<ChoiceOption> = ctx
                        .core
                        .player(self.owner)
                        .hand
                        .iter()
                        .copied()
                        .filter(|c| !traps.contains(&ctx.catalog.cards[*c as usize].suit))
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
                            prompt: "sxianjing_pick",
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
                        ctx.take_hand_card(self.owner, c);
                    }
                    ctx.core.player_mut(self.owner).sxianjing_traps.push(c);
                    Flow::Done
                }
                _ => Flow::Done,
            },
            // 入梦：弃非基本牌 → 目标弃基本牌或跳过出牌/弃牌阶段
            HostBehavior::SwdRumeng => match self.stage {
                HostStage::Act => {
                    // 触发帧即回合角色的出牌阶段开始帧——目标在此取定
                    let target = match ctx.res.frame(ctx.current_frame_id()).map(|f| &f.data) {
                        Some(EventData::Phase { player, .. }) => *player,
                        _ => return Flow::Done,
                    };
                    self.target = Some(target);
                    let options: Vec<ChoiceOption> = ctx
                        .core
                        .player(self.owner)
                        .hand
                        .iter()
                        .chain(ctx.core.player(self.owner).equips.iter())
                        .copied()
                        .filter(|c| !ctx.catalog.cards[*c as usize].kind.is_basic_kind())
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
                            prompt: "rumeng_discard",
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
                    // 弃置（手牌或装备区）
                    if ctx.core.player(self.owner).hand.contains(&card) {
                        ctx.discard_from_hand(self.owner, card);
                    } else if ctx.core.player(self.owner).equips.contains(&card) {
                        ctx.core
                            .player_mut(self.owner)
                            .equips
                            .retain(|c| *c != card);
                        ctx.to_discard(card);
                    } else {
                        return Flow::Done;
                    }
                    ctx.log(LogEvent::SkillTriggered {
                        skill: "入梦",
                        owner: self.owner,
                    });
                    let Some(target) = self.target else {
                        return Flow::Done;
                    };
                    let basics: Vec<ChoiceOption> = ctx
                        .core
                        .player(target)
                        .hand
                        .iter()
                        .copied()
                        .filter(|c| ctx.catalog.cards[*c as usize].kind.is_basic_kind())
                        .map(ChoiceOption::Card)
                        .collect();
                    self.stage = HostStage::TargetChoice;
                    Flow::WaitChoice {
                        observer: target,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectCards,
                            prompt: "rumeng_basic_or_skip",
                            options: basics,
                            min: 1,
                            max: 1,
                            allow_cancel: true,
                        },
                    }
                }
                HostStage::TargetChoice => {
                    let Some(target) = self.target else {
                        return Flow::Done;
                    };
                    match ctx.answer().first().copied() {
                        Some(ChoiceOption::Card(card)) => {
                            ctx.discard_from_hand(target, card);
                        }
                        _ => {
                            // 取消：跳过其出牌与弃牌阶段（上游 trigger.cancel
                            // + skip('phaseDiscard')）——立即收束当前出牌阶段，
                            // 弃牌阶段由跳位承载
                            ctx.core.player_mut(target).skip_discard_phase = true;
                            ctx.emit(EventData::Phase {
                                player: target,
                                phase: crate::event::Phase::Play,
                                begin: false,
                            });
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
