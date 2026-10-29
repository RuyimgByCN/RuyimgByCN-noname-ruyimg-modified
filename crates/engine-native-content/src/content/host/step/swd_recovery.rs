use super::super::*;

impl HostMachine {
    pub(super) fn step_swd_recovery(&mut self, ctx: &mut Ctx<'_>) -> Flow {
        match self.behavior {
            // 湛卢：SelectCard 弃一张黑桃牌（he 可取消=放弃）→ TargetChoice
            // 选至多 3 名已受伤角色 → 各回复 1
            HostBehavior::SwdZhanlu => match self.stage {
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
                                ctx.catalog.cards[*c as usize].suit,
                                crate::catalog::Suit::Spade
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
                            prompt: "zhanlu_discard",
                            options,
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
                            let pl = ctx.core.player(*p);
                            pl.hp < pl.max_hp as i32
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
                            prompt: "zhanlu_targets",
                            options,
                            min: 1,
                            max: 3,
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
                    for t in targets {
                        if ctx.core.player(t).alive {
                            ctx.recover(t, 1, None);
                        }
                    }
                    Flow::Done
                }
                _ => Flow::Done,
            },
            // 相惠：SelectCard 弃一张红色手牌 → 全体体力最少且已受伤角色
            // 各回复 1（目标面自动求取，无选择窗）
            HostBehavior::SwdXianghui => match self.stage {
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
                            prompt: "xianghui_discard",
                            options,
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
                    ctx.take_hand_card(self.owner, picked);
                    ctx.to_discard(picked);
                    ctx.log(LogEvent::CardDiscarded {
                        player: self.owner,
                        cards: vec![picked],
                    });
                    let min_hp = ctx
                        .core
                        .alive_players()
                        .map(|p| ctx.core.player(p).hp)
                        .min()
                        .unwrap_or(0);
                    let targets: Vec<PlayerId> = ctx
                        .core
                        .alive_players()
                        .filter(|p| {
                            let pl = ctx.core.player(*p);
                            pl.hp == min_hp && pl.hp < pl.max_hp as i32
                        })
                        .collect();
                    for t in targets {
                        ctx.recover(t, 1, None);
                    }
                    Flow::Done
                }
                _ => Flow::Done,
            },
            // 鸣凤：SelectCard 弃一张装备牌 → TargetChoice 任一存活角色
            // → 回复 1 + 摸 1
            HostBehavior::SwdFengming => match self.stage {
                HostStage::Act => {
                    let options: Vec<ChoiceOption> = ctx
                        .core
                        .player(self.owner)
                        .hand
                        .iter()
                        .chain(ctx.core.player(self.owner).equips.iter())
                        .copied()
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
                            prompt: "fengming_discard",
                            options,
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
                            prompt: "fengming_target",
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
                        let pl = ctx.core.player(t);
                        if pl.hp < pl.max_hp as i32 {
                            ctx.recover(t, 1, None);
                        }
                        ctx.draw_cards(t, 1);
                    }
                    Flow::Done
                }
                _ => Flow::Done,
            },
            // 神鼎·麦星面：确定性判定——红桃回 1/方块摸 1/梅花随机敌弃 1/
            // 黑桃无（内联口径同铁骑）
            HostBehavior::SwdShending => {
                let judged = ctx.draw_judge_card();
                let suit = ctx.catalog.cards[judged as usize].suit;
                ctx.to_discard(judged);
                match suit {
                    crate::catalog::Suit::Heart => {
                        let pl = ctx.core.player(self.owner);
                        if pl.hp < pl.max_hp as i32 {
                            ctx.recover(self.owner, 1, None);
                        }
                    }
                    crate::catalog::Suit::Diamond => {
                        ctx.draw_cards(self.owner, 1);
                    }
                    crate::catalog::Suit::Club => {
                        let mut pool: Vec<PlayerId> = ctx
                            .core
                            .alive_players()
                            .filter(|p| {
                                *p != self.owner
                                    && (!ctx.core.player(*p).hand.is_empty()
                                        || !ctx.core.player(*p).equips.is_empty())
                            })
                            .collect();
                        if !pool.is_empty() {
                            let j = ctx.core.rng.gen_range(pool.len() as u32) as usize;
                            let t = pool.swap_remove(j);
                            let mut he: Vec<CardId> =
                                ctx.core.player(t).hand.iter().copied().collect();
                            he.extend(ctx.core.player(t).equips.iter().copied());
                            let k = ctx.core.rng.gen_range(he.len() as u32) as usize;
                            let c = he[k];
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
                        }
                    }
                    crate::catalog::Suit::Spade => {}
                }
                Flow::Done
            }
            // 小莫雨·山小小面：锁定——回复 1（满血改摸 1）；每回合一次
            //（inst.once_used 消费——limits once_per_turn 走回合重置）
            HostBehavior::SwdXiaomoyu => {
                // 每回合一次由 resolution 层强制触发路径自动消费 once_used
                //（plan→start 前置检查），机器臂直接执行效果
                if !ctx.core.player(self.owner).alive {
                    return Flow::Done;
                }
                let pl = ctx.core.player(self.owner);
                if pl.hp < pl.max_hp as i32 {
                    ctx.recover(self.owner, 1, None);
                } else {
                    ctx.draw_cards(self.owner, 1);
                }
                Flow::Done
            }
            // 枕流触发面：off_court 红牌入手并回复 1（登记差异：按花色
            // 过滤取牌——若 off_court 有他技红色挂附可能误取）
            HostBehavior::SwdZhenjiu2 => {
                let red = ctx
                    .core
                    .player(self.owner)
                    .off_court
                    .iter()
                    .copied()
                    .find(|c| {
                        matches!(
                            ctx.catalog.cards[*c as usize].suit,
                            crate::catalog::Suit::Heart | crate::catalog::Suit::Diamond
                        )
                    });
                if let Some(c) = red {
                    ctx.core
                        .player_mut(self.owner)
                        .off_court
                        .retain(|x| *x != c);
                    ctx.core.player_mut(self.owner).hand.push(c);
                    ctx.log(LogEvent::CardsGiven {
                        from: self.owner,
                        to: self.owner,
                        cards: vec![c],
                    });
                    ctx.recover(self.owner, 1, None);
                }
                if let Some(def) = ctx.catalog.skill_by_name(SWD_ZHENJIU2_KEY) {
                    ctx.core.skills.remove(&(self.owner, def));
                }
                Flow::Done
            }
            // 兰芷：使用梅花牌后，体力不大于自己的已受伤角色（含自己）
            // 按座位序各回复 1（上游 list.sort(lib.sort.seat)；恒发动近似）。
            HostBehavior::SwdLanzhi => {
                let n = ctx.core.players.len() as PlayerId;
                let own_hp = ctx.core.player(self.owner).hp;
                for t in 0..n {
                    let p = ctx.core.player(t);
                    if p.alive && p.hp < p.max_hp as i32 && p.hp <= own_hp {
                        ctx.recover(t, 1, None);
                    }
                }
                Flow::Done
            }
            // 唤魂：确认 → SelectCard 弃一张 he → 濒死者判定（内联）：
            // 红→回 1；黑→弃置牌入手濒死者
            HostBehavior::SwdHuanhun => match self.stage {
                HostStage::Act => {
                    let Some(dp) = ctx
                        .res
                        .frame(ctx.current_frame_id())
                        .map(|f| &f.data)
                        .and_then(|data| match data {
                            crate::event::EventData::Dying { player, .. } => Some(*player),
                            _ => None,
                        })
                    else {
                        return Flow::Done;
                    };
                    self.target = Some(dp);
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
                            prompt: "huanhun_discard",
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
                    let Some(dp) = self.target else {
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
                    let judged = ctx.draw_judge_card();
                    let red = matches!(
                        ctx.catalog.cards[judged as usize].suit,
                        crate::catalog::Suit::Heart | crate::catalog::Suit::Diamond
                    );
                    ctx.to_discard(judged);
                    if red {
                        if ctx.core.player(dp).alive || ctx.core.player(dp).hp <= 0 {
                            ctx.recover(dp, 1, None);
                        }
                    } else if ctx.core.discard.contains(&picked) && ctx.core.player(dp).alive {
                        // 弃置牌仍在弃堆（未被中间效果取走）→ 濒死者获得
                        ctx.core.discard.retain(|x| *x != picked);
                        ctx.core.player_mut(dp).hand.push(picked);
                        ctx.log(LogEvent::CardsGiven {
                            from: self.owner,
                            to: dp,
                            cards: vec![picked],
                        });
                    }
                    self.target = None;
                    Flow::Done
                }
                _ => Flow::Done,
            },
            // 不死：濒死强制判定（内联口径同铁骑）——黑桃照常（落救援/
            // 死亡）；非黑桃体力回至 1+翻面（TurnOverBefore 帧经 commit
            // 含凶兽守卫/急冻解除）；unique 一次后移除实例
            HostBehavior::SwdBusi => {
                let judged = ctx.draw_judge_card();
                let spade = matches!(
                    ctx.catalog.cards[judged as usize].suit,
                    crate::catalog::Suit::Spade
                );
                ctx.to_discard(judged);
                if !spade {
                    let hp = ctx.core.player(self.owner).hp;
                    let need = (1 - hp).max(1) as u32;
                    ctx.recover(self.owner, need, None);
                    ctx.emit(EventData::TurnOverBefore {
                        player: self.owner,
                        cancelled: false,
                    });
                }
                if let Some(d) = ctx.catalog.skill_by_name("busi") {
                    ctx.core.skills.remove(&(self.owner, d));
                }
                Flow::Done
            }
            // 鬼眼·复活面：濒死强制回 1（recover 自清 dying 旗→救援机循
            // 环顶自然 DyingSaved）+ 移除鬼眼双技实例
            HostBehavior::SwdGuiyan2 => {
                ctx.recover(self.owner, 1, None);
                for name in ["guiyan", "swd_guiyan2"] {
                    if let Some(d) = ctx.catalog.skill_by_name(name) {
                        ctx.core.skills.remove(&(self.owner, d));
                    }
                }
                Flow::Done
            }
            // 墨剑：目标摸 1 + 自己回 1（未满血门）
            HostBehavior::SwdMojian => {
                if let Some(t) = self.target {
                    if ctx.core.player(t).alive {
                        ctx.draw_cards(t, 1);
                    }
                }
                let p = ctx.core.player(self.owner);
                if p.hp < p.max_hp as i32 {
                    ctx.recover(self.owner, 1, None);
                }
                Flow::Done
            }
            // 协律：确认门后回 1（未满血门）+ 弃置场上全部判定牌（座位序）
            HostBehavior::SwdXielv => {
                let p = ctx.core.player(self.owner);
                if p.alive && p.hp < p.max_hp as i32 {
                    ctx.recover(self.owner, 1, None);
                }
                let judge_cards: Vec<(PlayerId, CardId)> = (0..ctx.core.players.len())
                    .flat_map(|i| {
                        let pl = ctx.core.player(i as PlayerId);
                        pl.judge
                            .iter()
                            .map(|c| (i as PlayerId, *c))
                            .collect::<Vec<_>>()
                    })
                    .collect();
                for (pl, c) in judge_cards {
                    ctx.core.player_mut(pl).judge.retain(|x| *x != c);
                    ctx.to_discard(c);
                    ctx.log(LogEvent::CardDiscarded {
                        player: pl,
                        cards: vec![c],
                    });
                }
                Flow::Done
            }
            // 花镜：回 1 + 摸 1（受伤者判定面：上游回 1 无判定——纯回摸）
            HostBehavior::SwdHuajing => {
                let p = ctx.core.player(self.owner);
                if p.hp < p.max_hp as i32 {
                    ctx.recover(self.owner, 1, None);
                }
                ctx.draw_cards(self.owner, 1);
                Flow::Done
            }
            // 守印：置限定位 + 自翻面 + 全员回复至满（座位序）
            HostBehavior::SwdShouyin => {
                ctx.core.player_mut(self.owner).shouyin_used = true;
                ctx.emit(EventData::TurnOverBefore {
                    player: self.owner,
                    cancelled: false,
                });
                ctx.log(LogEvent::SkillTriggered {
                    skill: "守印",
                    owner: self.owner,
                });
                for p in ctx.core.seat_order_from(self.owner) {
                    let (hp, max) = {
                        let pl = ctx.core.player(p);
                        (pl.hp, pl.max_hp)
                    };
                    if pl_alive(ctx, p) && hp < max as i32 {
                        ctx.recover(p, (max as i32 - hp) as u32, None);
                    }
                }
                Flow::Done
            }
            _ => unreachable!("behavior routed to the wrong step module"),
        }
    }
}
