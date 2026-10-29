//! 轩辕剑（swd）装备授予技能与卡牌窗口（神偷、御符录、狼哥怪异、鼠槌、玉蝉换形、神秘果）。

use super::super::*;

impl HostMachine {
    pub(super) fn step_swd_equipment_skills(&mut self, ctx: &mut Ctx<'_>) -> Flow {
        match self.behavior {
            // 神秘果窗口（卡牌面窗口机批）：确认后——发射载体神秘果用牌
            //（提交段自取+Shenmiguo 臂弃置=上游 useCard 同簿记），随后
            // 原体自弃堆上桌面重发射（登记差异：上游凭空复制同花色点数
            // 新实体，原生原体复用——弃堆在再结算窗口内少一张观测差）。
            // 确认即置位每阶段一次旗（上游 shenmiguo2 临时技：再结算牌
            // 的用牌后窗口被同一旗挡住=循环守卫）。
            HostBehavior::SwdShenmiguoWindow => match self.stage {
                HostStage::Act => {
                    let (x_card, x_targets) = {
                        let u = ctx.current_card_use().expect("card use frame");
                        (u.card, u.targets.clone())
                    };
                    let Some(fruit) = ctx
                        .core
                        .player(self.owner)
                        .hand
                        .iter()
                        .copied()
                        .find(|c| ctx.catalog.card_kind(*c) == CardKind::Shenmiguo)
                    else {
                        return Flow::Done;
                    };
                    self.card = Some(fruit);
                    self.cards = vec![x_card];
                    self.targets = x_targets;
                    ctx.core.turn.shenmiguo_used = true;
                    ctx.emit(EventData::CardUsed(crate::event::CardUseData {
                        card: fruit,
                        player: self.owner,
                        targets: Vec::new(),
                        nature: crate::event::DamageNature::Normal,
                        use_as: None,
                        cancelled: false,
                    }));
                    self.stage = HostStage::ShenmiguoRestage;
                    Flow::Yield
                }
                HostStage::ShenmiguoRestage => {
                    let Some(x_card) = self.cards.first().copied() else {
                        return Flow::Done;
                    };
                    // 原体自弃堆上桌面（处理区）；不在弃堆（特殊流转）则放弃
                    if ctx.core.discard.contains(&x_card) {
                        ctx.core.discard.retain(|c| *c != x_card);
                        ctx.core.table.push(x_card);
                    } else if !ctx.core.table.contains(&x_card) {
                        return Flow::Done;
                    }
                    let kind = ctx.catalog.card_kind(x_card);
                    let nature = match kind {
                        CardKind::FireSha => crate::event::DamageNature::Fire,
                        CardKind::LeiSha => crate::event::DamageNature::Thunder,
                        _ => crate::event::DamageNature::Normal,
                    };
                    ctx.emit(EventData::CardUsed(crate::event::CardUseData {
                        card: x_card,
                        player: self.owner,
                        targets: self.targets.clone(),
                        nature,
                        use_as: None,
                        cancelled: false,
                    }));
                    self.cards.clear();
                    self.targets.clear();
                    self.card = None;
                    Flow::Done
                }
                _ => Flow::Done,
            },
            // 鼠槌（equip5 窗口机批）：凤略 Yield 循环同型——选攻击范围
            // 内目标（sha_range_free 链同 engine 出杀口径）→ 至多 3 张
            // 逐张自动用出（每次重扫手中首张杀=上游 getCards("h")[0]
            // 确定性口径）；终局若链上任一杀造成伤害（commit_damage 钩
            // 置位 shuchui_hit）则摸 1。cards 复用为剩余次数计数。
            HostBehavior::SwdShuchuiUse => match self.stage {
                // 首步：目标询问（挂起前 stage 前移到消费阶段——active_ask
                // 铁律：快照 stage=挂起期间等待的下一阶段）
                HostStage::Act => {
                    let sha_range_free = ctx
                        .catalog
                        .skill_by_name("jiying")
                        .map(|d| ctx.core.has_skill(self.owner, d))
                        .unwrap_or(false);
                    let options: Vec<ChoiceOption> = ctx
                        .core
                        .alive_players()
                        .filter(|p| {
                            *p != self.owner
                                && (sha_range_free
                                    || ctx.core.computed_distance(ctx.catalog, self.owner, *p)
                                        <= ctx.core.attack_range(ctx.catalog, self.owner))
                        })
                        .map(ChoiceOption::Player)
                        .collect();
                    if options.is_empty() {
                        return Flow::Done;
                    }
                    self.stage = HostStage::ShuchuiTarget;
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectPlayers,
                            prompt: "shuchui_target",
                            options,
                            min: 1,
                            max: 1,
                            allow_cancel: false,
                        },
                    }
                }
                HostStage::ShuchuiTarget => {
                    if let Some(ChoiceOption::Player(t)) = ctx.answer().first().cloned() {
                        if t != self.owner && ctx.core.player(t).alive {
                            self.target = Some(t);
                        }
                    }
                    let Some(t) = self.target else {
                        return Flow::Done;
                    };
                    // 链头武装：伤害钩置位窗开（commit_damage 消费）
                    ctx.core.turn.shuchui_armed = true;
                    ctx.core.turn.shuchui_hit = false;
                    self.cards = vec![0, 0, 0];
                    self.stage = HostStage::ShuchuiTargetLoop;
                    self.shuchui_loop(ctx, t)
                }
                HostStage::ShuchuiTargetLoop => {
                    self.shuchui_loop(ctx, self.target.expect("loop stage implies target"))
                }
                _ => Flow::Done,
            },
            // 玉蝉换形: 手牌中每张玉蝉实体轮换为八卦序下一 kind 的实体
            //（牌堆/弃堆确定性发现首张；登记差异上游随机+init 原地变形）
            HostBehavior::SwdYuchanSwap => {
                const ORDER: [CardKind; 8] = [
                    CardKind::Yuchanqian,
                    CardKind::Yuchankun,
                    CardKind::Yuchanzhen,
                    CardKind::Yuchanxun,
                    CardKind::Yuchankan,
                    CardKind::Yuchanli,
                    CardKind::Yuchangen,
                    CardKind::Yuchandui,
                ];
                // 收集「换出牌 → 换入牌」对（按牌 id；不记索引——`to_discard`
                // 现在会按牌区唯一性移动牌，索引在循环中会失效）
                let mut swaps: Vec<(CardId, CardId)> = Vec::new();
                for c in ctx.core.player(self.owner).hand.iter().rev() {
                    let Some(pos) = ctx
                        .catalog
                        .card_kind(*c)
                        .yuchan_auto_view_as()
                        .and_then(|_| ORDER.iter().position(|k| k == &ctx.catalog.card_kind(*c)))
                    else {
                        continue;
                    };
                    let want = ORDER[(pos + 1) % 8];
                    if let Some(newc) = ctx
                        .core
                        .deck
                        .iter()
                        .chain(ctx.core.discard.iter())
                        .copied()
                        .find(|x| ctx.catalog.card_kind(*x) == want)
                    {
                        swaps.push((*c, newc));
                    }
                }
                for (oldc, newc) in swaps {
                    // 旧实体从手牌移出入弃堆（守恒），新实体从来源区移除后入手
                    ctx.take_hand_card(self.owner, oldc);
                    ctx.to_discard(oldc);
                    if let Some(dpos) = ctx.core.deck.iter().position(|x| *x == newc) {
                        ctx.core.deck.remove(dpos);
                    } else if let Some(dpos) = ctx.core.discard.iter().position(|x| *x == newc) {
                        ctx.core.discard.remove(dpos);
                    }
                    let hand = &mut ctx.core.player_mut(self.owner).hand;
                    if let Some(pos) = hand.iter().position(|c| *c == oldc) {
                        hand[pos] = newc;
                    } else {
                        // 旧牌已由 to_discard 移出：新牌直接入手（换形语义不变）
                        hand.push(newc);
                    }
                    ctx.log(LogEvent::CardsGiven {
                        from: self.owner,
                        to: self.owner,
                        cards: vec![newc],
                    });
                }
                Flow::Done
            }
            HostBehavior::SwdShentouEquipSkill => match self.stage {
                HostStage::Act => {
                    // 潜偷（swd 第三批，规则 v92）：出牌阶段限一次，弃一张
                    // 手牌为代价 → 判定梅花则失败 → 成功则获得手牌数比你多
                    // 的目标一张随机手牌（上游 shentou chooseCard cost +
                    // judge 梅花失败 + randomGet 同口径）
                    let nh = ctx.core.player(self.owner).hand.len();
                    let richer = ctx
                        .core
                        .alive_players()
                        .any(|p| p != self.owner && ctx.core.player(p).hand.len() > nh);
                    if nh == 0 || !richer {
                        // 无代价可付或无人手牌更多：技能空发（上游 filter）
                        return Flow::Done;
                    }
                    let options: Vec<ChoiceOption> = ctx
                        .core
                        .player(self.owner)
                        .hand
                        .iter()
                        .copied()
                        .map(ChoiceOption::Card)
                        .collect();
                    self.stage = HostStage::SelectCard;
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectCards,
                            prompt: SWD_SHENTOU_COST,
                            options,
                            min: 1,
                            max: 1,
                            allow_cancel: false,
                        },
                    }
                }
                HostStage::SelectCard => {
                    // 消费代价牌 → 判定（梅花失败）→ 成功偷牌
                    let cost = ctx.answer().first().and_then(|o| match o {
                        ChoiceOption::Card(c) => Some(*c),
                        _ => None,
                    });
                    if let Some(c) = cost.filter(|c| ctx.core.player(self.owner).hand.contains(c)) {
                        ctx.discard_from_hand(self.owner, c);
                        ctx.log(LogEvent::CardDiscarded {
                            player: self.owner,
                            cards: vec![c],
                        });
                    }
                    let judged = ctx.draw_judge_card();
                    let club = matches!(
                        ctx.catalog.cards[judged as usize].suit,
                        crate::catalog::Suit::Club
                    );
                    ctx.log(LogEvent::Judged {
                        player: self.owner,
                        card: judged,
                    });
                    ctx.to_discard(judged);
                    if club {
                        return Flow::Done;
                    }
                    // 成功：随机一名手牌更多的其他角色 → 随机一张手牌
                    let cands: Vec<crate::ids::PlayerId> = ctx
                        .core
                        .alive_players()
                        .filter(|p| {
                            *p != self.owner
                                && ctx.core.player(*p).hand.len()
                                    > ctx.core.player(self.owner).hand.len()
                        })
                        .collect();
                    if !cands.is_empty() {
                        let v = cands[ctx.core.rng.gen_range(cands.len() as u32) as usize];
                        let n = ctx.core.player(v).hand.len();
                        let i = ctx.core.rng.gen_range(n as u32) as usize;
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
                _ => Flow::Done,
            },
            HostBehavior::SwdYufuluEquipSkill => match self.stage {
                // 御夫录（swd 第十二批，规则 v96）：三段——Act 问武器
                //（手牌/装备区 equip1）→ SelectCard 消费并问目标 →
                // TargetChoice 目标受 1 点伤害并获得武器牌（上游
                // damage + gain 同序；dieThen 仍获得登记差异）。
                HostStage::Act => {
                    let is_weapon = |c: CardId| {
                        ctx.catalog.card_slot_of(ctx.core, c)
                            == Some(crate::catalog::EquipSlot::Weapon)
                    };
                    let has_weapon = ctx
                        .core
                        .player(self.owner)
                        .hand
                        .iter()
                        .chain(ctx.core.player(self.owner).equips.iter())
                        .copied()
                        .any(is_weapon);
                    let others = ctx.core.alive_players().any(|p| p != self.owner);
                    if !has_weapon || !others {
                        return Flow::Done;
                    }
                    let mut options: Vec<ChoiceOption> = ctx
                        .core
                        .player(self.owner)
                        .hand
                        .iter()
                        .copied()
                        .filter(|c| is_weapon(*c))
                        .map(ChoiceOption::Card)
                        .collect();
                    options.extend(
                        ctx.core
                            .player(self.owner)
                            .equips
                            .iter()
                            .copied()
                            .filter(|c| is_weapon(*c))
                            .map(ChoiceOption::Card),
                    );
                    self.stage = HostStage::SelectCard;
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectCards,
                            prompt: SWD_YUFULU_WEAPON,
                            options,
                            min: 1,
                            max: 1,
                            allow_cancel: false,
                        },
                    }
                }
                HostStage::SelectCard => {
                    // 记录所选武器（保留在原区域，上游 prepare:"give" 同
                    // 口径——卡牌在 content 执行时才移动）→ 问目标
                    let w = ctx.answer().first().and_then(|o| match o {
                        ChoiceOption::Card(c) => Some(*c),
                        _ => None,
                    });
                    let w = w.filter(|c| {
                        ctx.core.player(self.owner).hand.contains(c)
                            || ctx.core.player(self.owner).equips.contains(c)
                    });
                    let Some(weapon) = w else {
                        return Flow::Done;
                    };
                    self.card = Some(weapon);
                    let options: Vec<ChoiceOption> = ctx
                        .core
                        .alive_players()
                        .filter(|p| *p != self.owner)
                        .map(ChoiceOption::Player)
                        .collect();
                    self.stage = HostStage::TargetChoice;
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectPlayers,
                            prompt: SWD_YUFULU_TARGET,
                            options,
                            min: 1,
                            max: 1,
                            allow_cancel: false,
                        },
                    }
                }
                HostStage::TargetChoice => {
                    let Some(weapon) = self.card.take() else {
                        return Flow::Done;
                    };
                    let target = ctx.answer().first().and_then(|o| match o {
                        ChoiceOption::Player(p) => Some(*p),
                        _ => None,
                    });
                    let Some(t) = target.filter(|p| ctx.core.player(*p).alive) else {
                        return Flow::Done;
                    };
                    // 此刻才移动武器（手牌→直接取出；装备区→取出并撤销
                    // 装备技），保持存档核算各询问挂起态区域完整
                    if ctx.core.player(self.owner).hand.contains(&weapon) {
                        ctx.core
                            .player_mut(self.owner)
                            .hand
                            .retain(|x| *x != weapon);
                    } else if ctx.core.player(self.owner).equips.contains(&weapon) {
                        ctx.core
                            .player_mut(self.owner)
                            .equips
                            .retain(|x| *x != weapon);
                        crate::kernel::revoke_equip_skills(
                            ctx.core,
                            ctx.catalog,
                            self.owner,
                            weapon,
                        );
                    } else {
                        return Flow::Done;
                    }
                    // 上游 content：damage 后 gain
                    ctx.emit(EventData::Damage(crate::machine::damage_data_with(
                        Some(self.owner),
                        t,
                        1,
                        crate::event::DamageNature::Normal,
                        None,
                    )));
                    ctx.core.player_mut(t).hand.push(weapon);
                    ctx.log(LogEvent::CardsGiven {
                        from: self.owner,
                        to: t,
                        cards: vec![weapon],
                    });
                    Flow::Done
                }
                _ => Flow::Done,
            },
            HostBehavior::SwdLangeguaiyiEquipSkill => match self.stage {
                // 蓝格怪衣「龙帆」（swd 第十五批，规则 v99）：判定按花色
                // 分支——红桃回 1 体力（ctx.recover 满血饱和为既有路径，
                // 上游 recover 同钳制）、方片摸 1、梅花随机敌人随机弃
                // 一张、黑桃无事。上游 judge 的 suit 提示函数为 AI 排序
                // （不迁移）；判定牌入弃堆（Shentou 同款）。
                HostStage::Act => {
                    let judged = ctx.draw_judge_card();
                    let suit = ctx.catalog.cards[judged as usize].suit;
                    ctx.log(LogEvent::Judged {
                        player: self.owner,
                        card: judged,
                    });
                    ctx.to_discard(judged);
                    use crate::catalog::Suit;
                    match suit {
                        Suit::Heart => {
                            ctx.recover(self.owner, 1, None);
                        }
                        Suit::Diamond => {
                            ctx.draw_cards(self.owner, 1);
                        }
                        Suit::Club => {
                            // 候选=其他存活且（手牌∪装备区）非空（上游
                            // he filter 同口径）；随机取一。上游
                            // randomDiscard() 默认区不可从归档核验（现有
                            // 调用点均显式 "h"）——登记差异：手牌优先、
                            // 无手牌退随机装备（离装撤销装备技）。
                            let mut cands: Vec<crate::ids::PlayerId> = ctx
                                .core
                                .alive_players()
                                .filter(|p| {
                                    *p != self.owner
                                        && (!ctx.core.player(*p).hand.is_empty()
                                            || !ctx.core.player(*p).equips.is_empty())
                                })
                                .collect();
                            if !cands.is_empty() {
                                let i = ctx.core.rng.gen_range(cands.len() as u32) as usize;
                                let t = cands.swap_remove(i);
                                if !ctx.core.player(t).hand.is_empty() {
                                    let n = ctx.core.player(t).hand.len();
                                    let j = ctx.core.rng.gen_range(n as u32) as usize;
                                    let c = ctx.core.player_mut(t).hand.remove(j);
                                    ctx.to_discard(c);
                                    ctx.log(LogEvent::CardDiscarded {
                                        player: t,
                                        cards: vec![c],
                                    });
                                } else {
                                    let n = ctx.core.player(t).equips.len();
                                    let j = ctx.core.rng.gen_range(n as u32) as usize;
                                    let c = ctx.core.player_mut(t).equips.remove(j);
                                    crate::kernel::revoke_equip_skills(ctx.core, ctx.catalog, t, c);
                                    ctx.to_discard(c);
                                    ctx.log(LogEvent::CardDiscarded {
                                        player: t,
                                        cards: vec![c],
                                    });
                                }
                            }
                        }
                        Suit::Spade => {}
                    }
                    Flow::Done
                }
                _ => Flow::Done,
            },
            _ => unreachable!("behavior routed to the wrong step module"),
        }
    }
}
