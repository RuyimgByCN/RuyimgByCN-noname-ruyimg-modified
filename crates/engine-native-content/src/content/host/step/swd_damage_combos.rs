use super::super::*;

impl HostMachine {
    pub(super) fn step_swd_damage_combos(&mut self, ctx: &mut Ctx<'_>) -> Flow {
        match self.behavior {
            // 星坠·慕月面：SelectCard 弃一张牌（定类别）→ TargetChoice 有
            // he 目标 → 目标弃同类别一张或受 1 伤
            HostBehavior::SwdXingzhui => match self.stage {
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
                            prompt: "xingzhui_discard",
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
                    self.cards = vec![picked]; // 类别锚（get.type trick 近似：装备/非装备二分登记差异上游 basic/trick/equip 三分）
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
                            prompt: "xingzhui_target",
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
                    self.target = Some(t);
                    // 同类别=同装备槽有无可弃近似（上游 get.type 同类）：
                    // 装备牌锚→装备候选；非装备锚→非装备候选
                    let anchor = self.cards.first().copied();
                    let anchor_is_equip = anchor
                        .map(|c| ctx.catalog.card_kind(c).is_equip())
                        .unwrap_or(false);
                    let anchor_is_basic = anchor
                        .map(|c| ctx.catalog.card_kind(c).is_basic_kind())
                        .unwrap_or(false);
                    let candidates: Vec<CardId> = ctx
                        .core
                        .player(t)
                        .hand
                        .iter()
                        .chain(ctx.core.player(t).equips.iter())
                        .copied()
                        .filter(|c| {
                            let k = ctx.catalog.card_kind(*c);
                            if anchor_is_equip {
                                k.is_equip()
                            } else if anchor_is_basic {
                                k.is_basic_kind()
                            } else {
                                !k.is_equip() && !k.is_basic_kind()
                            }
                        })
                        .collect();
                    if candidates.is_empty() {
                        ctx.emit(EventData::Damage(crate::machine::damage_data_with(
                            Some(self.owner),
                            t,
                            1,
                            crate::event::DamageNature::Normal,
                            None,
                        )));
                        return Flow::Done;
                    }
                    self.stage = HostStage::CardsAsk;
                    Flow::WaitChoice {
                        observer: t,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectCards,
                            prompt: "xingzhui_same_type",
                            options: candidates.iter().copied().map(ChoiceOption::Card).collect(),
                            min: 1,
                            max: 1,
                            allow_cancel: false,
                        },
                    }
                }
                HostStage::CardsAsk => {
                    let Some(c) = ctx.answer().iter().find_map(|o| match o {
                        ChoiceOption::Card(c) => Some(*c),
                        _ => None,
                    }) else {
                        return Flow::Done;
                    };
                    let Some(t) = self.target else {
                        return Flow::Done;
                    };
                    if ctx.core.player(t).hand.contains(&c) {
                        ctx.discard_from_hand(t, c);
                    } else if ctx.core.player(t).equips.contains(&c) {
                        ctx.core.player_mut(t).equips.retain(|x| *x != c);
                        if ctx.catalog.card_kind(c).is_equip() {
                            crate::kernel::revoke_equip_skills(ctx.core, ctx.catalog, t, c);
                        }
                        ctx.to_discard(c);
                        ctx.log(LogEvent::CardDiscarded {
                            player: t,
                            cards: vec![c],
                        });
                    }
                    Flow::Done
                }
                _ => Flow::Done,
            },
            // 义绝（武将面批 86）：CompareTargetPick 目标选拼点牌 →
            // ComparePlayerPick 己方选牌 → 比点：赢=目标置封印（本回合
            // 基本/锦囊失效）+受 1 伤；输=目标已受伤回 1
            HostBehavior::SwdYijue => match self.stage {
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
                            prompt: "yijue_target",
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
                    self.stage = HostStage::CompareTargetPick;
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
                            prompt: "yijue_target_pick",
                            options,
                            min: 1,
                            max: 1,
                            allow_cancel: false,
                        },
                    }
                }
                HostStage::CompareTargetPick => {
                    let Some(ChoiceOption::Card(t_card)) = ctx.answer().first().copied() else {
                        return Flow::Done;
                    };
                    self.cards = vec![t_card];
                    self.stage = HostStage::ComparePlayerPick;
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
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectCards,
                            prompt: "yijue_player_pick",
                            options,
                            min: 1,
                            max: 1,
                            allow_cancel: false,
                        },
                    }
                }
                HostStage::ComparePlayerPick => {
                    let Some(ChoiceOption::Card(p_card)) = ctx.answer().first().copied() else {
                        return Flow::Done;
                    };
                    let Some(t) = self.target else {
                        return Flow::Done;
                    };
                    let Some(t_card) = self.cards.first().copied() else {
                        return Flow::Done;
                    };
                    // 拼点牌入弃堆（上游拼点结算同口径）
                    if ctx.core.player(self.owner).hand.contains(&p_card) {
                        ctx.discard_from_hand(self.owner, p_card);
                    }
                    if ctx.core.player(t).hand.contains(&t_card) {
                        ctx.discard_from_hand(t, t_card);
                    }
                    let pr = ctx.catalog.cards[p_card as usize].rank;
                    let tr = ctx.catalog.cards[t_card as usize].rank;
                    if pr > tr {
                        // 赢：封印（本回合基本/锦囊用牌失效——fengyin_active
                        // 同族三面查询；登记差异：原生封印在目标出牌校验
                        // 已含 h 位置）+受 1 伤
                        ctx.core.player_mut(t).fengyin_active = true;
                        ctx.emit(EventData::Damage(crate::machine::damage_data_with(
                            Some(self.owner),
                            t,
                            1,
                            crate::event::DamageNature::Normal,
                            None,
                        )));
                    } else {
                        // 输：目标已受伤回 1
                        let pl = ctx.core.player(t);
                        if pl.hp < pl.max_hp as i32 {
                            ctx.recover(t, 1, None);
                        }
                    }
                    Flow::Done
                }
                _ => Flow::Done,
            },
            // 拼点（deferred 消解批 独孤宁珂）：弃一张手牌，令两名角色拼点：
            // 赢者获得该牌、输者受赢者 1 伤（拼点牌两目标自选；登记差异：
            // 目标/拼点牌座位序与确定性近似）
            HostBehavior::SwdNlianji => match self.stage {
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
                            prompt: "nlianji_throw",
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
                    self.cards = vec![c];
                    self.stage = HostStage::TargetChoice;
                    Flow::Yield
                }
                HostStage::TargetChoice => {
                    let options: Vec<ChoiceOption> = ctx
                        .core
                        .alive_players()
                        .filter(|p| *p != self.owner && !ctx.core.player(*p).hand.is_empty())
                        .map(ChoiceOption::Player)
                        .collect();
                    if options.is_empty() {
                        return Flow::Done;
                    }
                    self.stage = HostStage::CardsAsk;
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectPlayers,
                            prompt: "nlianji_target0",
                            options,
                            min: 1,
                            max: 1,
                            allow_cancel: true,
                        },
                    }
                }
                HostStage::CardsAsk => {
                    let Some(ChoiceOption::Player(t0)) = ctx.answer().first().copied() else {
                        return Flow::Done;
                    };
                    self.target = Some(t0);
                    let options: Vec<ChoiceOption> = ctx
                        .core
                        .alive_players()
                        .filter(|p| {
                            *p != self.owner && *p != t0 && !ctx.core.player(*p).hand.is_empty()
                        })
                        .map(ChoiceOption::Player)
                        .collect();
                    if options.is_empty() {
                        return Flow::Done;
                    }
                    self.stage = HostStage::CompareTargetPick;
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectPlayers,
                            prompt: "nlianji_target1",
                            options,
                            min: 1,
                            max: 1,
                            allow_cancel: true,
                        },
                    }
                }
                HostStage::CompareTargetPick => {
                    let Some(ChoiceOption::Player(t1)) = ctx.answer().first().copied() else {
                        return Flow::Done;
                    };
                    self.targets = vec![t1];
                    let Some(t0) = self.target else {
                        return Flow::Done;
                    };
                    let options: Vec<ChoiceOption> = ctx
                        .core
                        .player(t0)
                        .hand
                        .iter()
                        .copied()
                        .map(ChoiceOption::Card)
                        .collect();
                    if options.is_empty() {
                        return Flow::Done;
                    }
                    self.stage = HostStage::ComparePlayerPick;
                    Flow::WaitChoice {
                        observer: t0,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectCards,
                            prompt: "nlianji_t0_pick",
                            options,
                            min: 1,
                            max: 1,
                            allow_cancel: false,
                        },
                    }
                }
                HostStage::ComparePlayerPick => {
                    let Some(ChoiceOption::Card(t0_card)) = ctx.answer().first().copied() else {
                        return Flow::Done;
                    };
                    let Some(t0) = self.target else {
                        return Flow::Done;
                    };
                    let Some(t1) = self.targets.first().copied() else {
                        return Flow::Done;
                    };
                    // t1 拼点牌：座位序首个手牌（登记差异：自选→首个）
                    let Some(t1_card) = ctx.core.player(t1).hand.first().copied() else {
                        return Flow::Done;
                    };
                    let Some(throw_card) = self.cards.first().copied() else {
                        return Flow::Done;
                    };
                    if ctx.core.player(t0).hand.contains(&t0_card) {
                        ctx.discard_from_hand(t0, t0_card);
                    }
                    if ctx.core.player(t1).hand.contains(&t1_card) {
                        ctx.discard_from_hand(t1, t1_card);
                    }
                    let r0 = ctx.catalog.cards[t0_card as usize].rank;
                    let r1 = ctx.catalog.cards[t1_card as usize].rank;
                    if r0 == r1 {
                        return Flow::Done;
                    }
                    let (winner, loser) = if r0 > r1 { (t0, t1) } else { (t1, t0) };
                    ctx.give_hand_card(self.owner, winner, throw_card);
                    ctx.emit(EventData::Damage(crate::machine::damage_data_with(
                        Some(winner),
                        loser,
                        1,
                        crate::event::DamageNature::Normal,
                        None,
                    )));
                    Flow::Done
                }
                _ => Flow::Done,
            },
            // 蛇魂（deferred 消解批 山小小）：弃 1–4 张手牌并选一名有牌角色：
            // 其弃等量 he 牌，若其中有与弃牌同花色者则其受 1 伤
            // （登记差异：花色互异约束未强制——任意 1–4 张）
            HostBehavior::SwdShehun => match self.stage {
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
                            prompt: "shehun_discard",
                            options,
                            min: 1,
                            max: 4,
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
                    if picked.is_empty() {
                        return Flow::Done;
                    }
                    self.cards = picked;
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
                            prompt: "shehun_target",
                            options,
                            min: 1,
                            max: 1,
                            allow_cancel: true,
                        },
                    }
                }
                HostStage::TargetChoice => {
                    let Some(ChoiceOption::Player(t)) = ctx.answer().first().copied() else {
                        return Flow::Done;
                    };
                    self.target = Some(t);
                    let n = self.cards.len();
                    let he: Vec<CardId> = ctx
                        .core
                        .player(t)
                        .hand
                        .iter()
                        .chain(ctx.core.player(t).equips.iter())
                        .copied()
                        .collect();
                    if he.is_empty() {
                        return Flow::Done;
                    }
                    let take = n.min(he.len());
                    self.stage = HostStage::CardsAsk;
                    Flow::WaitChoice {
                        observer: t,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectCards,
                            prompt: "shehun_target_discard",
                            options: he.into_iter().map(ChoiceOption::Card).collect(),
                            min: take,
                            max: take,
                            allow_cancel: false,
                        },
                    }
                }
                HostStage::CardsAsk => {
                    let t_discard: Vec<CardId> = ctx
                        .answer()
                        .iter()
                        .filter_map(|o| match o {
                            ChoiceOption::Card(c) => Some(*c),
                            _ => None,
                        })
                        .collect();
                    let Some(t) = self.target else {
                        return Flow::Done;
                    };
                    let own_suits: Vec<crate::catalog::Suit> = self
                        .cards
                        .iter()
                        .map(|c| ctx.catalog.cards[*c as usize].suit)
                        .collect();
                    let own = std::mem::take(&mut self.cards);
                    for c in own {
                        if ctx.core.player(self.owner).hand.contains(&c) {
                            ctx.discard_from_hand(self.owner, c);
                        }
                    }
                    let mut damage = false;
                    for c in t_discard {
                        if own_suits.contains(&ctx.catalog.cards[c as usize].suit) {
                            damage = true;
                        }
                        if ctx.core.player(t).hand.contains(&c) {
                            ctx.discard_from_hand(t, c);
                        } else {
                            ctx.core.player_mut(t).equips.retain(|x| *x != c);
                            if ctx.catalog.card_kind(c).is_equip() {
                                crate::kernel::revoke_equip_skills(ctx.core, ctx.catalog, t, c);
                            }
                            ctx.to_discard(c);
                            ctx.log(LogEvent::CardDiscarded {
                                player: t,
                                cards: vec![c],
                            });
                        }
                    }
                    if damage {
                        ctx.emit(EventData::Damage(crate::machine::damage_data_with(
                            Some(self.owner),
                            t,
                            1,
                            crate::event::DamageNature::Normal,
                            None,
                        )));
                    }
                    Flow::Done
                }
                _ => Flow::Done,
            },
            _ => unreachable!("behavior routed to the wrong step module"),
        }
    }
}
