//! 轩辕剑（swd）阵法牌效果（天覆阵、七星阵、鸟翔阵、长蛇阵、破阵诀）。

use super::super::*;

impl HostMachine {
    pub(super) fn step_swd_zhenfa_cards(&mut self, ctx: &mut Ctx<'_>) -> Flow {
        match self.behavior {
            // 天覆阵（zhenfa，规则 v200/T19a）：AOE 逐大国目标强制弃 1 张
            // he（targets 队列；目标自选牌 min 1 不可取消；self.target=当
            // 前询问目标）。全部结算完后本体入弃堆。
            HostBehavior::SwdTianfuzhenCard => match self.stage {
                HostStage::Act => {
                    if self.targets.is_empty() {
                        if let Some(c) = self.card {
                            ctx.to_discard(c);
                            ctx.core.table.retain(|x| *x != c);
                            self.card = None;
                        }
                        return Flow::Done;
                    }
                    let t = self.targets.remove(0);
                    self.target = Some(t);
                    let options: Vec<ChoiceOption> = ctx
                        .core
                        .player(t)
                        .hand
                        .iter()
                        .copied()
                        .chain(ctx.core.player(t).equips.iter().copied())
                        .map(ChoiceOption::Card)
                        .collect();
                    if options.is_empty() {
                        return Flow::Yield;
                    }
                    self.stage = HostStage::SelectCard;
                    Flow::WaitChoice {
                        observer: t,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectCards,
                            prompt: "swd_tianfuzhen_discard",
                            options,
                            min: 1,
                            max: 1,
                            allow_cancel: false,
                        },
                    }
                }
                HostStage::SelectCard => {
                    if let Some(ChoiceOption::Card(c)) = ctx.answer().first().copied() {
                        let t = self.target.unwrap_or(self.owner);
                        if ctx.core.player(t).hand.contains(&c) {
                            ctx.discard_from_hand(t, c);
                        } else if let Some(pos) =
                            ctx.core.player(t).equips.iter().position(|x| *x == c)
                        {
                            ctx.core.player_mut(t).equips.remove(pos);
                            crate::kernel::revoke_equip_skills(ctx.core, ctx.catalog, t, c);
                            ctx.to_discard(c);
                        }
                        ctx.log(LogEvent::CardDiscarded {
                            player: t,
                            cards: vec![c],
                        });
                    }
                    self.stage = HostStage::Act;
                    Flow::Yield
                }
                _ => Flow::Done,
            },
            // 七星阵（zhenfa，规则 v200/T19a）：阶段一 Act——逐围攻者弃 1
            //（SelectCard 询问，self.target=围攻者）；队列空后阶段二——
            // 视为对围攻者用杀（响应闪或 1 伤；登记差异：虚拟杀简化，无
            // 加成/不可无懈额外面），阶段二队列存 self.cards。本体入弃堆。
            HostBehavior::SwdQixingzhenCard => match self.stage {
                HostStage::Act => {
                    if !self.targets.is_empty() {
                        let t = self.targets.remove(0);
                        self.target = Some(t);
                        let options: Vec<ChoiceOption> = ctx
                            .core
                            .player(t)
                            .hand
                            .iter()
                            .copied()
                            .chain(ctx.core.player(t).equips.iter().copied())
                            .map(ChoiceOption::Card)
                            .collect();
                        if options.is_empty() {
                            return Flow::Yield;
                        }
                        self.stage = HostStage::SelectCard;
                        return Flow::WaitChoice {
                            observer: t,
                            spec: ChoiceSpec {
                                kind: ChoiceKind::SelectCards,
                                prompt: "swd_qixingzhen_discard",
                                options,
                                min: 1,
                                max: 1,
                                allow_cancel: false,
                            },
                        };
                    }
                    if self.cards.is_empty() {
                        if let Some(c) = self.card {
                            ctx.to_discard(c);
                            ctx.core.table.retain(|x| *x != c);
                            self.card = None;
                        }
                        return Flow::Done;
                    }
                    let t = self.cards.remove(0) as u8;
                    self.target = Some(t);
                    let options: Vec<ChoiceOption> = ctx
                        .core
                        .player(t)
                        .hand
                        .iter()
                        .copied()
                        .filter(|c| ctx.catalog.card_kind(*c) == CardKind::Shan)
                        .map(ChoiceOption::Card)
                        .collect();
                    if options.is_empty() {
                        // 无闪：直接结算 1 点伤害（机关凤同款收口）
                        ctx.emit(EventData::Damage(crate::machine::damage_data_with(
                            Some(self.owner),
                            t,
                            1,
                            crate::event::DamageNature::Normal,
                            None,
                        )));
                        self.stage = HostStage::Act;
                        return Flow::Yield;
                    }
                    self.stage = HostStage::TargetChoice;
                    Flow::WaitChoice {
                        observer: t,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectCards,
                            prompt: "swd_qixingzhen_shan",
                            options,
                            min: 0,
                            max: 1,
                            allow_cancel: true,
                        },
                    }
                }
                HostStage::SelectCard => {
                    if let Some(ChoiceOption::Card(c)) = ctx.answer().first().copied() {
                        let t = self.target.unwrap_or(self.owner);
                        if ctx.core.player(t).hand.contains(&c) {
                            ctx.discard_from_hand(t, c);
                        } else if let Some(pos) =
                            ctx.core.player(t).equips.iter().position(|x| *x == c)
                        {
                            ctx.core.player_mut(t).equips.remove(pos);
                            crate::kernel::revoke_equip_skills(ctx.core, ctx.catalog, t, c);
                            ctx.to_discard(c);
                        }
                        ctx.log(LogEvent::CardDiscarded {
                            player: t,
                            cards: vec![c],
                        });
                    }
                    self.stage = HostStage::Act;
                    Flow::Yield
                }
                HostStage::TargetChoice => {
                    let t = self.target.unwrap_or(self.owner);
                    if matches!(ctx.answer().first(), Some(ChoiceOption::Card(_))) {
                        if let Some(ChoiceOption::Card(c)) = ctx.answer().first().copied() {
                            ctx.discard_from_hand(t, c);
                            ctx.log(LogEvent::CardDiscarded {
                                player: t,
                                cards: vec![c],
                            });
                        }
                    } else {
                        ctx.emit(EventData::Damage(crate::machine::damage_data_with(
                            Some(self.owner),
                            t,
                            1,
                            crate::event::DamageNature::Normal,
                            None,
                        )));
                    }
                    self.stage = HostStage::Act;
                    Flow::Yield
                }
                _ => Flow::Done,
            },
            // 鸟翔阵（zhenfa，规则 v200/T19a）：逐合法目标响应闪或受 1 伤
            //（targets 队列；TargetChoice 询问，取消=伤害）。本体入弃堆。
            HostBehavior::SwdNiaoxiangzhenCard => match self.stage {
                HostStage::Act => {
                    if self.targets.is_empty() {
                        if let Some(c) = self.card {
                            ctx.to_discard(c);
                            ctx.core.table.retain(|x| *x != c);
                            self.card = None;
                        }
                        return Flow::Done;
                    }
                    let t = self.targets.remove(0);
                    self.target = Some(t);
                    self.stage = HostStage::TargetChoice;
                    let options: Vec<ChoiceOption> = ctx
                        .core
                        .player(t)
                        .hand
                        .iter()
                        .copied()
                        .filter(|c| ctx.catalog.card_kind(*c) == CardKind::Shan)
                        .map(ChoiceOption::Card)
                        .collect();
                    Flow::WaitChoice {
                        observer: t,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectCards,
                            prompt: "swd_niaoxiangzhen_shan",
                            options,
                            min: 0,
                            max: 1,
                            allow_cancel: true,
                        },
                    }
                }
                HostStage::TargetChoice => {
                    let t = self.target.unwrap_or(self.owner);
                    if matches!(ctx.answer().first(), Some(ChoiceOption::Card(_))) {
                        if let Some(ChoiceOption::Card(c)) = ctx.answer().first().copied() {
                            ctx.discard_from_hand(t, c);
                            ctx.log(LogEvent::CardDiscarded {
                                player: t,
                                cards: vec![c],
                            });
                        }
                    } else {
                        ctx.emit(EventData::Damage(crate::machine::damage_data_with(
                            Some(self.owner),
                            t,
                            1,
                            crate::event::DamageNature::Normal,
                            None,
                        )));
                    }
                    self.stage = HostStage::Act;
                    Flow::Yield
                }
                _ => Flow::Done,
            },
            // 长蛇阵（zhenfa，规则 v201/T19b）：队列中→全队列（含自己）各
            // 摸 1；否则与距离最近的友方换座成队列（swapSeat）；本体入弃堆。
            HostBehavior::SwdChangshezhenCard => {
                let inline_now = ctx.core.inline(ctx.catalog, self.owner);
                let prev_of_owner = ctx.core.alive_neighbors(self.owner).1;
                if inline_now {
                    for q in 0..ctx.core.players.len() as u8 {
                        if ctx.core.inline_with(ctx.catalog, self.owner, q) {
                            ctx.start_draw(q, 1);
                        }
                    }
                } else if prev_of_owner.is_some() {
                    // 友方按**顺座次序**排序（NC01：顺序语义用 seat_order_from；
                    // 上游按座次轮转取最近）
                    let mut friends: Vec<(u8, u8)> = (0..ctx.core.players.len() as u8)
                        .filter(|q| {
                            *q != self.owner
                                && ctx.core.player(*q).alive
                                && ctx.core.is_friend_of(ctx.catalog, self.owner, *q)
                        })
                        .map(|q| (q, ctx.core.seat_steps_from(self.owner, q)))
                        .collect();
                    friends.sort_by_key(|(_, d)| *d);
                    if let Some((nearest, _)) = friends.first().copied() {
                        let next_of_owner = ctx.core.next_alive(self.owner);
                        ctx.core.swap_seat(nearest, next_of_owner);
                    }
                }
                if let Some(c) = self.card {
                    ctx.to_discard(c);
                    ctx.core.table.retain(|x| *x != c);
                    self.card = None;
                }
                Flow::Done
            }
            // 破阵诀（zhenfa，规则 v201/T19b）：n 次随机座位交换（n=存活
            // 数，上游 swapSeat(random, random) ×n 同口径）；本体入弃堆。
            HostBehavior::SwdPozhenjueCard => {
                let n = ctx.core.alive_players().count();
                for _ in 0..n {
                    let seats: Vec<u8> = ctx.core.alive_players().collect();
                    if seats.len() < 2 {
                        break;
                    }
                    let i = ctx.core.rng.gen_range(seats.len() as u32) as usize;
                    let j = ctx.core.rng.gen_range(seats.len() as u32) as usize;
                    ctx.core.swap_seat(seats[i], seats[j]);
                }
                if let Some(c) = self.card {
                    ctx.to_discard(c);
                    ctx.core.table.retain(|x| *x != c);
                    self.card = None;
                }
                Flow::Done
            }
            _ => unreachable!("behavior routed to the wrong step module"),
        }
    }
}
