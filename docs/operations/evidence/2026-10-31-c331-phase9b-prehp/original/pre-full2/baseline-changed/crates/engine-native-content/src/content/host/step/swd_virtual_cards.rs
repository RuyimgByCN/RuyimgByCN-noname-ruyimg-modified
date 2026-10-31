//! 轩辕剑（swd）宿主技：弃牌后视为使用杀或群体锦囊（连打、焰羽、乱击、诛害）。

use super::super::*;

impl HostMachine {
    pub(super) fn step_swd_virtual_cards(&mut self, ctx: &mut Ctx<'_>) -> Flow {
        match self.behavior {
            // 连打：SelectCard 弃一张牌（he 可取消=放弃）→ 虚拟杀对原目
            // 标再结算（凤略直发口径；每回合旗置位=循环守卫）
            HostBehavior::SwdLianda => match self.stage {
                HostStage::Act => {
                    let Some(u) = ctx.current_card_use().ok() else {
                        return Flow::Done;
                    };
                    self.targets = u.targets.clone();
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
                            prompt: "lianda_discard",
                            options: pool.iter().copied().map(ChoiceOption::Card).collect(),
                            min: 0,
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
                    ctx.core.turn.lianda_used = true;
                    let targets = std::mem::take(&mut self.targets);
                    if targets.is_empty() || targets.iter().any(|t| !ctx.core.player(*t).alive) {
                        return Flow::Done;
                    }
                    ctx.use_virtual_card(self.owner, CardKind::Sha, targets);
                    Flow::Done
                }
                _ => Flow::Done,
            },
            // 焰羽·少微面：SelectCards 弃两张红色牌 → 虚拟炽羽袭（全体其他）
            HostBehavior::SwdHuangyu => match self.stage {
                HostStage::Act => {
                    let reds: Vec<ChoiceOption> = ctx
                        .core
                        .player(self.owner)
                        .hand
                        .iter()
                        .chain(ctx.core.player(self.owner).equips.iter())
                        .copied()
                        .filter(|c| {
                            matches!(
                                ctx.catalog.cards[*c as usize].suit,
                                crate::catalog::Suit::Heart | crate::catalog::Suit::Diamond
                            )
                        })
                        .map(ChoiceOption::Card)
                        .collect();
                    if reds.len() < 2 {
                        return Flow::Done;
                    }
                    self.stage = HostStage::SelectCard;
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectCards,
                            prompt: "huangyu_discard",
                            options: reds,
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
                    if picks.len() < 2 {
                        return Flow::Done;
                    }
                    for c in picks {
                        let in_hand = ctx.core.player(self.owner).hand.contains(&c);
                        if in_hand {
                            ctx.take_hand_card(self.owner, c);
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
                        }
                        ctx.to_discard(c);
                        ctx.log(LogEvent::CardDiscarded {
                            player: self.owner,
                            cards: vec![c],
                        });
                    }
                    // 虚拟炽羽袭：全体其他存活（座位序，同 AOE push 口径）
                    let targets: Vec<PlayerId> = ctx
                        .core
                        .seat_order_from(ctx.core.next_alive(self.owner))
                        .into_iter()
                        .filter(|p| *p != self.owner)
                        .collect();
                    if targets.is_empty() {
                        return Flow::Done;
                    }
                    ctx.use_virtual_card(self.owner, CardKind::Chiyuxi, targets);
                    Flow::Done
                }
                _ => Flow::Done,
            },
            // 乱击·多鹏面：SelectCards 弃两张同花色手牌 → 虚拟万箭齐发
            //（AOE 全体其他存活，座位序）
            HostBehavior::SwdLuanji => match self.stage {
                HostStage::Act => {
                    let pairs = same_suit_pair_options(ctx.core, self.owner);
                    if pairs.is_empty() {
                        return Flow::Done;
                    }
                    self.stage = HostStage::SelectCard;
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectCards,
                            prompt: "luanji_discard",
                            options: pairs,
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
                    if picks.len() != 2
                        || !upstream_same_suit(ctx, &picks)
                        || picks
                            .iter()
                            .any(|c| !ctx.core.player(self.owner).hand.contains(c))
                    {
                        return Flow::Done;
                    }
                    for c in picks {
                        ctx.discard_from_hand(self.owner, c);
                    }
                    let targets: Vec<PlayerId> = ctx
                        .core
                        .seat_order_from(ctx.core.next_alive(self.owner))
                        .into_iter()
                        .filter(|p| *p != self.owner)
                        .collect();
                    if targets.is_empty() {
                        return Flow::Done;
                    }
                    ctx.use_virtual_card(self.owner, CardKind::ArcheryAttack, targets);
                    Flow::Done
                }
                _ => Flow::Done,
            },
            // 诛害：选一张手中的【杀】→ 对结束阶段角色使用（实体牌）
            HostBehavior::SwdZhuhai => match self.stage {
                HostStage::Act => {
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
                        .copied()
                        .filter(|c| ctx.catalog.cards[*c as usize].kind.is_sha_like())
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
                            prompt: "zhuhai_sha",
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
                    if !ctx.core.player(self.owner).hand.contains(&card)
                        || !ctx.core.player(target).alive
                    {
                        return Flow::Done;
                    }
                    ctx.log(LogEvent::SkillTriggered {
                        skill: "诛害",
                        owner: self.owner,
                    });
                    ctx.emit(EventData::CardUsed(crate::machine::card_use(
                        card,
                        self.owner,
                        vec![target],
                    )));
                    self.stage = HostStage::ShuchuiTargetLoop;
                    Flow::Yield
                }
                _ => Flow::Done,
            },
            _ => unreachable!("behavior routed to the wrong step module"),
        }
    }
}
