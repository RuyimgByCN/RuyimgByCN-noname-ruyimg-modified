//! 轩辕剑（swd）宿主技：付牌后指定目标并取牌、交牌或复制（鬼眼、孵鸟、分舵、魅惑、运筹、幻形）。

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
