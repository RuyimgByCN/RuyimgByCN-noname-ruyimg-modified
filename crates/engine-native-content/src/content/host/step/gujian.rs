use super::super::*;

impl HostMachine {
    pub(super) fn step_gujian(&mut self, ctx: &mut Ctx<'_>) -> Flow {
        match self.behavior {
            HostBehavior::GujianShujinsanCard => match self.stage {
                HostStage::SelectCard => {
                    // 目标选任意张手牌（至少 1，可取消；取消则无事发生）
                    let Some(t) = self.target else {
                        return Flow::Done;
                    };
                    let options: Vec<ChoiceOption> = ctx
                        .core
                        .player(t)
                        .hand
                        .iter()
                        .copied()
                        .map(ChoiceOption::Card)
                        .collect();
                    if options.is_empty() || !ctx.core.player(t).alive {
                        if let Some(c) = self.card {
                            ctx.to_discard(c);
                            ctx.core.table.retain(|x| *x != c);
                        }
                        return Flow::Done;
                    }
                    let n = options.len();
                    self.stage = HostStage::Act;
                    Flow::WaitChoice {
                        observer: t,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectCards,
                            prompt: GJ_SHUJINSAN_DISCARD,
                            options,
                            min: 1,
                            max: n,
                            allow_cancel: true,
                        },
                    }
                }
                HostStage::Act => {
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
                    if !picked.is_empty() {
                        for c in &picked {
                            ctx.core.player_mut(t).hand.retain(|x| *x != *c);
                        }
                        ctx.log(LogEvent::CardDiscarded {
                            player: t,
                            cards: picked.clone(),
                        });
                        ctx.draw_cards(t, picked.len());
                    }
                    if let Some(c) = self.card {
                        ctx.to_discard(c);
                        ctx.core.table.retain(|x| *x != c);
                    }
                    Flow::Done
                }
                _ => Flow::Done,
            },
            HostBehavior::GujianMianliCard => {
                // 目标摸一张，然后受 1 点伤害（登记差异：上游为失去体力）
                let Some(t) = self.target else {
                    return Flow::Done;
                };
                if ctx.core.player(t).alive {
                    ctx.draw_cards(t, 1);
                    // 失去体力（逐技对照批三 v77 消除「伤害替代」登记差异）
                    ctx.emit(EventData::LoseHp {
                        player: t,
                        amount: 1,
                    });
                }
                if let Some(c) = self.card {
                    ctx.to_discard(c);
                    ctx.core.table.retain(|x| *x != c);
                }
                Flow::Done
            }
            HostBehavior::GujianFeibiaoCard => match self.stage {
                HostStage::SelectCard => {
                    // 有黑色手牌 → 选一张弃置（可取消，取消则受伤）；
                    // 否则直接受伤
                    let Some(t) = self.target else {
                        return Flow::Done;
                    };
                    let finish = |ctx: &mut Ctx<'_>, card: Option<CardId>| {
                        if let Some(c) = card {
                            ctx.to_discard(c);
                            ctx.core.table.retain(|x| *x != c);
                        }
                    };
                    if !ctx.core.player(t).alive {
                        finish(ctx, self.card);
                        return Flow::Done;
                    }
                    let options: Vec<ChoiceOption> = ctx
                        .core
                        .player(t)
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
                        ctx.emit(EventData::Damage(crate::machine::damage_data_with(
                            Some(self.owner),
                            t,
                            1,
                            crate::event::DamageNature::Normal,
                            self.card,
                        )));
                        finish(ctx, self.card);
                        return Flow::Done;
                    }
                    self.stage = HostStage::Act;
                    Flow::WaitChoice {
                        observer: t,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectCards,
                            prompt: GJ_FEIBIAO_DISCARD,
                            options,
                            min: 1,
                            max: 1,
                            allow_cancel: true,
                        },
                    }
                }
                HostStage::Act => {
                    let Some(t) = self.target else {
                        return Flow::Done;
                    };
                    // 空应答 = 取消（allow_cancel）→ 失去 1 点体力
                    //（v77 原语对齐：上游 loseHp 同口径）
                    let chosen = ctx.answer().first().copied();
                    match chosen {
                        Some(ChoiceOption::Card(c)) => {
                            ctx.core.player_mut(t).hand.retain(|x| *x != c);
                            ctx.log(LogEvent::CardDiscarded {
                                player: t,
                                cards: vec![c],
                            });
                        }
                        _ => {
                            ctx.emit(EventData::LoseHp {
                                player: t,
                                amount: 1,
                            });
                        }
                    }
                    if let Some(c) = self.card {
                        ctx.to_discard(c);
                        ctx.core.table.retain(|x| *x != c);
                    }
                    Flow::Done
                }
                _ => Flow::Done,
            },
            HostBehavior::GujianFoodTanhuadong => {
                // 出牌阶段结束：摸 1；倒计时扣减，归零移除状态
                ctx.draw_cards(self.owner, 1);
                food_tick(ctx, self.owner, &self.def_key);
                Flow::Done
            }
            HostBehavior::GujianFoodYougeng => {
                // 阶段开始（判定阶段替位准备阶段）：已受伤且体力值为
                // 全场最少或之一 → 回复 1；倒计时扣减（无论是否触发
                // 回复，登记差异：与上游同点扣减）
                let hp = ctx.core.player(self.owner).hp;
                let max_hp = ctx.core.player(self.owner).max_hp as i32;
                let min_hp = ctx
                    .core
                    .alive_players()
                    .all(|p| ctx.core.player(p).hp >= hp);
                if hp < max_hp && min_hp {
                    ctx.recover(self.owner, 1, None);
                }
                food_tick(ctx, self.owner, &self.def_key);
                Flow::Done
            }
            HostBehavior::GujianFoodGrant => {
                // 使用食物牌：给目标安装对应监听技实例（rounds=剩余
                // 回合）并弃置食物牌（食物无懈不可响应，无窗口）
                let Some(t) = self.target else {
                    return Flow::Done;
                };
                let Some(c) = self.card else {
                    return Flow::Done;
                };
                let (listener, rounds) = match ctx.catalog.card_kind(c) {
                    CardKind::Tanhuadong => (GJ_FOOD_TANHUADONG_KEY, GJ_FOOD_ROUNDS_TANHUADONG),
                    CardKind::Yougeng => (GJ_FOOD_YOUGENG_KEY, GJ_FOOD_ROUNDS_YOUGENG),
                    CardKind::Qingtuan => (GJ_FOOD_QINGTUAN_KEY, GJ_FOOD_ROUNDS_DEFAULT),
                    CardKind::Yuanbaorou => (GJ_FOOD_YUANBAOROU_KEY, GJ_FOOD_ROUNDS_DEFAULT),
                    CardKind::Gudonggeng => (GJ_FOOD_GUDONGGENG_KEY, GJ_FOOD_ROUNDS_DEFAULT),
                    CardKind::Mapodoufu => (GJ_FOOD_MAPODOUFU_KEY, GJ_FOOD_ROUNDS_DEFAULT),
                    CardKind::Chunbing => (GJ_FOOD_CHUNBING_KEY, GJ_FOOD_ROUNDS_CHUNBING),
                    CardKind::Liyutang => (GJ_FOOD_LIYUTANG_KEY, GJ_FOOD_ROUNDS_LIYUTANG),
                    CardKind::Molicha => (GJ_FOOD_MOLICHA_KEY, GJ_FOOD_ROUNDS_DEFAULT),
                    CardKind::Xiajiao => (GJ_FOOD_XIAJIAO_KEY, GJ_FOOD_ROUNDS_XIAJIAO),
                    CardKind::Mizhilianou => (GJ_FOOD_MIZHLIANOU_KEY, GJ_FOOD_ROUNDS_MIZHLIANOU),
                    CardKind::Luyugeng => (GJ_FOOD_LUYUGENG_KEY, GJ_FOOD_ROUNDS_DEFAULT),
                    CardKind::Ziyangdan => (GJ_FOOD_ZIYANGDAN_KEY, GJ_FOOD_ROUNDS_DEFAULT),
                    CardKind::Yunvyuanshen => {
                        (GJ_YUNVYUANSHEN_LISTENER_KEY, GJ_FOOD_ROUNDS_DEFAULT)
                    }
                    _ => return Flow::Done,
                };
                let Some(def) = ctx.catalog.skill_by_name(listener) else {
                    return Flow::Done;
                };
                ctx.insert_skill(
                    (t, def),
                    crate::state::SkillInstance {
                        owner: t,
                        enabled: true,
                        once_used: false,
                        expires_end_of_turn: false,
                        rounds,
                    },
                );
                // 紫阳丹：授予同时获得 3 点护甲（可叠加，登记差异：
                // 上游每阶段衰减，原生为每回合结束 -1）
                if ctx.catalog.card_kind(c) == CardKind::Ziyangdan {
                    ctx.change_hujia(t, GJ_ZIYANGDAN_HUJIA as i32);
                }
                ctx.to_discard(c);
                ctx.core.table.retain(|x| *x != c);
                Flow::Done
            }
            HostBehavior::GujianFoodQingtuan => {
                // 回合内使用首张杀（once_per_turn 门控）：摸 1
                ctx.draw_cards(self.owner, 1);
                food_tick(ctx, self.owner, &self.def_key);
                Flow::Done
            }
            HostBehavior::GujianFoodYuanbaorou => {
                // 元宝肉（古剑食物，规则 v48）：出牌阶段可额外使用一张杀
                // （每回合一次）。上游 cardUsable num+1 与 mark 回合递减；效果
                // 在杀次数校验（engine.rs 2339 行读 once_used、2501 行用后置位）
                // 与目标校验（engine.rs 3329 行），宿主无机器步骤。
                Flow::Done
            }
            HostBehavior::GujianFoodGudonggeng => {
                // 受到杀伤害：伤害 -1；一次性状态，触发即移除
                let d = ctx.current_damage_mut();
                d.amount = d.amount.saturating_sub(1);
                food_remove(ctx, self.owner, &self.def_key);
                Flow::Done
            }
            HostBehavior::GujianFoodMapodoufu => {
                // 结束阶段：随机一名有牌其他角色随机弃一张牌（手牌+装备，
                // 上游 list.randomGet() + he.randomGet()；逐技对照批四
                // v78 种子 RNG 均匀抽取；敌我阵营面按原型惯例取其他角色）
                let cands: Vec<crate::ids::PlayerId> = ctx
                    .core
                    .seat_order_from(ctx.core.turn.current)
                    .into_iter()
                    .filter(|p| {
                        *p != self.owner
                            && ctx.core.player(*p).alive
                            && (!ctx.core.player(*p).hand.is_empty()
                                || !ctx.core.player(*p).equips.is_empty())
                    })
                    .collect();
                let victim = if cands.is_empty() {
                    None
                } else {
                    Some(cands[ctx.core.rng.gen_range(cands.len() as u32) as usize])
                };
                if let Some(v) = victim {
                    let he_len = ctx.core.player(v).hand.len() + ctx.core.player(v).equips.len();
                    let pick = ctx.core.rng.gen_range(he_len as u32) as usize;
                    let hand_card = ctx.core.player(v).hand.get(pick).copied();
                    match hand_card {
                        Some(c) => {
                            ctx.discard_from_hand(v, c);
                            ctx.log(LogEvent::CardDiscarded {
                                player: v,
                                cards: vec![c],
                            });
                        }
                        None => {
                            // pick 落在装备区：下标减去手牌数
                            let ei = pick - ctx.core.player(v).hand.len();
                            if let Some(c) = ctx.core.player(v).equips.get(ei).copied() {
                                ctx.core.player_mut(v).equips.retain(|x| *x != c);
                                crate::kernel::revoke_equip_skills(ctx.core, ctx.catalog, v, c);
                                ctx.to_discard(c);
                                ctx.log(LogEvent::CardDiscarded {
                                    player: v,
                                    cards: vec![c],
                                });
                            }
                        }
                    }
                }
                food_tick(ctx, self.owner, &self.def_key);
                Flow::Done
            }
            HostBehavior::GujianFoodChunbing => Flow::Done,
            HostBehavior::GujianFoodLiyutang => {
                // 阶段开始：体力值为全场最少或之一 → 获得 1 点护甲
                let hp = ctx.core.player(self.owner).hp;
                let min_hp = ctx
                    .core
                    .alive_players()
                    .all(|p| ctx.core.player(p).hp >= hp);
                if min_hp {
                    ctx.change_hujia(self.owner, 1);
                }
                food_tick(ctx, self.owner, &self.def_key);
                Flow::Done
            }
            HostBehavior::GujianFoodMolicha => Flow::Done,
            HostBehavior::GujianFoodXiajiao => match self.stage {
                HostStage::Act => {
                    // 摸牌阶段：额外摸 1（固定摸牌后），然后弃 1
                    ctx.draw_cards(self.owner, 1);
                    let options: Vec<ChoiceOption> = ctx
                        .core
                        .player(self.owner)
                        .hand
                        .iter()
                        .copied()
                        .map(ChoiceOption::Card)
                        .collect();
                    if options.is_empty() || !ctx.core.player(self.owner).alive {
                        food_tick(ctx, self.owner, &self.def_key);
                        return Flow::Done;
                    }
                    self.stage = HostStage::SelectCard;
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectCards,
                            prompt: GJ_XIAJIAO_DISCARD,
                            options,
                            min: 1,
                            max: 1,
                            allow_cancel: false,
                        },
                    }
                }
                HostStage::SelectCard => {
                    // 弃 1（必选）后倒计时扣减
                    if let Some(ChoiceOption::Card(c)) = ctx.answer().first().copied() {
                        ctx.discard_from_hand(self.owner, c);
                        ctx.log(LogEvent::CardDiscarded {
                            player: self.owner,
                            cards: vec![c],
                        });
                    }
                    food_tick(ctx, self.owner, &self.def_key);
                    Flow::Done
                }
                _ => Flow::Done,
            },
            HostBehavior::GujianFoodMizhilianou => Flow::Done,
            HostBehavior::GujianFoodLuyugeng => match self.stage {
                HostStage::Act => {
                    // 出牌阶段限一次：弃一张基本牌，获得牌堆顶牌
                    //（登记差异：上游为发现牌随机五选一）
                    let options: Vec<ChoiceOption> = ctx
                        .core
                        .player(self.owner)
                        .hand
                        .iter()
                        .copied()
                        .filter(|c| {
                            matches!(
                                ctx.catalog.card_kind(*c),
                                CardKind::Sha | CardKind::Shan | CardKind::Tao | CardKind::Jiu
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
                            prompt: GJ_LUYUGENG_COST,
                            options,
                            min: 1,
                            max: 1,
                            allow_cancel: false,
                        },
                    }
                }
                HostStage::SelectCard => {
                    if let Some(ChoiceOption::Card(c)) = ctx.answer().first().copied() {
                        ctx.discard_from_hand(self.owner, c);
                        ctx.log(LogEvent::CardDiscarded {
                            player: self.owner,
                            cards: vec![c],
                        });
                        ctx.draw_cards(self.owner, 1);
                    }
                    food_tick(ctx, self.owner, &self.def_key);
                    Flow::Done
                }
                _ => Flow::Done,
            },
            HostBehavior::GujianFoodZiyangdan => {
                // A2 H22：上游 player phaseBegin＝自己回合开始护甲 -1（原锚
                // 自己结束阶段开始＝登记差异对齐收紧）；倒计时扣减，归零移除
                ctx.change_hujia(self.owner, -1);
                food_tick(ctx, self.owner, &self.def_key);
                Flow::Done
            }
            HostBehavior::GujianYunvyuanshen => {
                // 下次濒死：回复 1 点体力（recover 清 dying 标记，
                // 濒死救援机器检测 hp>0 直接退出）并移除状态
                ctx.recover(self.owner, 1, None);
                food_remove(ctx, self.owner, &self.def_key);
                Flow::Done
            }
            HostBehavior::GujianFoodShihuifenBan => Flow::Done,
            HostBehavior::GujianShihuifenCard => match self.stage {
                HostStage::Act => {
                    // 目标有闪 → 出闪询问；无闪 → 置禁用状态
                    let Some(t) = self.target else {
                        return Flow::Done;
                    };
                    let shans: Vec<ChoiceOption> = ctx
                        .core
                        .player(t)
                        .hand
                        .iter()
                        .copied()
                        .filter(|c| ctx.catalog.card_kind(*c) == CardKind::Shan)
                        .map(ChoiceOption::Card)
                        .collect();
                    if shans.is_empty() || !ctx.core.player(t).alive {
                        self.grant_shihuifen_ban(ctx, t);
                        if let Some(c) = self.card {
                            ctx.to_discard(c);
                            ctx.core.table.retain(|x| *x != c);
                        }
                        return Flow::Done;
                    }
                    let mut options = vec![ChoiceOption::Pass];
                    options.extend(shans);
                    self.stage = HostStage::TargetChoice;
                    Flow::WaitChoice {
                        observer: t,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::RespondCard,
                            prompt: "respond_shan",
                            options,
                            min: 1,
                            max: 1,
                            allow_cancel: false,
                        },
                    }
                }
                HostStage::TargetChoice => {
                    let Some(t) = self.target else {
                        return Flow::Done;
                    };
                    match ctx.answer().first().copied() {
                        Some(ChoiceOption::Card(c)) => {
                            ctx.discard_from_hand(t, c);
                            ctx.emit(EventData::CardResponded { card: c, player: t });
                        }
                        _ => self.grant_shihuifen_ban(ctx, t),
                    }
                    if let Some(c) = self.card {
                        ctx.to_discard(c);
                        ctx.core.table.retain(|x| *x != c);
                    }
                    Flow::Done
                }
                _ => Flow::Done,
            },
            HostBehavior::GujianLiufengsanCard => {
                // 目标从牌堆获得两张实体闪（登记差异：虚拟牌生成改
                // 实体牌获得；牌堆无闪则无事）
                let Some(t) = self.target else {
                    return Flow::Done;
                };
                if ctx.core.player(t).alive {
                    let mut gained = Vec::new();
                    for _ in 0..2 {
                        let pos = ctx
                            .core
                            .deck
                            .iter()
                            .rposition(|c| ctx.catalog.card_kind(*c) == CardKind::Shan);
                        match pos {
                            Some(i) => {
                                let c = ctx.core.deck.remove(i);
                                ctx.core.player_mut(t).hand.push(c);
                                gained.push(c);
                            }
                            None => break,
                        }
                    }
                    if !gained.is_empty() {
                        ctx.emit(EventData::CardsDrawn {
                            player: t,
                            cards: gained,
                        });
                    }
                }
                if let Some(c) = self.card {
                    ctx.to_discard(c);
                    ctx.core.table.retain(|x| *x != c);
                }
                Flow::Done
            }
            HostBehavior::GujianMutouEquipSkill => Flow::Done,
            HostBehavior::GujianYuhengEquipSkill => {
                // 随机一名有黑桃手牌的其他角色 → 其随机一张黑桃手牌交给你
                //（上游 list.randomGet() + randomGet()；逐技对照批四 v78
                // 种子 RNG 均匀抽取；升级机制仍登记暂缓）
                let cands: Vec<crate::ids::PlayerId> = ctx
                    .core
                    .alive_players()
                    .filter(|p| {
                        *p != self.owner
                            && ctx.core.player(*p).hand.iter().any(|c| {
                                ctx.catalog.cards[*c as usize].suit == crate::catalog::Suit::Spade
                            })
                    })
                    .collect();
                if !cands.is_empty() {
                    let v = cands[ctx.core.rng.gen_range(cands.len() as u32) as usize];
                    let spades: Vec<usize> = ctx
                        .core
                        .player(v)
                        .hand
                        .iter()
                        .enumerate()
                        .filter(|(_, c)| {
                            ctx.catalog.cards[**c as usize].suit == crate::catalog::Suit::Spade
                        })
                        .map(|(i, _)| i)
                        .collect();
                    let i = spades[ctx.core.rng.gen_range(spades.len() as u32) as usize];
                    let c = ctx.core.player_mut(v).hand.remove(i);
                    ctx.core.player_mut(self.owner).hand.push(c);
                    ctx.log(LogEvent::CardsGiven {
                        from: v,
                        to: self.owner,
                        cards: vec![c],
                    });
                }
                Flow::Done
            }
            _ => unreachable!("behavior routed to the wrong step module"),
        }
    }
}
