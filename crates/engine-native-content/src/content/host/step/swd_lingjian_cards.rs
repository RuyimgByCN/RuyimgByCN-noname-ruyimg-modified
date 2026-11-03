//! 轩辕剑（swd）灵剑牌效果（旋风之刃、中行虎甲、星体改造、时间回溯、紧急冷冻、圣休号角、隐秘离场）。

use super::super::*;

impl HostMachine {
    pub(super) fn step_swd_lingjian_cards(&mut self, ctx: &mut Ctx<'_>) -> Flow {
        match self.behavior {
            // 旋风之刃（灵剑，规则 v198）：目标随机弃 1 张 he；本体入弃堆。
            HostBehavior::SwdXuanfengzhirenCard => {
                if let Some(t) = self.target {
                    if ctx.core.player(t).alive {
                        let mut pool: Vec<CardId> = ctx
                            .core
                            .player(t)
                            .hand
                            .iter()
                            .copied()
                            .chain(ctx.core.player(t).equips.iter().copied())
                            .collect();
                        if !pool.is_empty() {
                            let k = ctx.core.rng.gen_range(pool.len() as u32) as usize;
                            let c = pool.swap_remove(k);
                            if ctx.core.player(t).hand.contains(&c) {
                                ctx.discard_from_hand(t, c);
                            } else if let Some(pos) =
                                ctx.core.player(t).equips.iter().position(|x| *x == c)
                            {
                                ctx.core.player_mut(t).equips.remove(pos);
                                crate::kernel::revoke_equip_skills(ctx.core, ctx.catalog, t, c);
                                ctx.to_discard(c);
                                // 装备弃置也发弃置事件（上游 loseEnd 含
                                // 装备区牌；机关妖术补偿依赖此事件）
                                ctx.emit(EventData::CardsDiscarded {
                                    player: t,
                                    cards: vec![c],
                                });
                            }
                            ctx.log(LogEvent::CardDiscarded {
                                player: t,
                                cards: vec![c],
                            });
                        }
                    }
                }
                if let Some(c) = self.card {
                    ctx.to_discard(c);
                    ctx.core.table.retain(|x| *x != c);
                    self.card = None;
                }
                Flow::Done
            }
            // 中行虎甲（灵剑，规则 v198）：目标非 isMin——从牌堆+弃堆发现防
            // 具 Kind 实体装配（登记差异：上游装备定义池凭空造），再随机弃
            // 1 张手牌；本体入弃堆。
            HostBehavior::SwdZhongxinghujiaCard => {
                if let Some(t) = self.target {
                    if ctx.core.player(t).alive {
                        let mut pool: Vec<CardId> = ctx
                            .core
                            .deck
                            .iter()
                            .chain(ctx.core.discard.iter())
                            .copied()
                            .filter(|x| ctx.catalog.card_kind(*x) == CardKind::Armor)
                            .collect();
                        let n = pool.len();
                        for i in (1..n).rev() {
                            let j = ctx.core.rng.gen_range(i as u32 + 1) as usize;
                            pool.swap(i, j);
                        }
                        if let Some(x) = pool.first().copied() {
                            if let Some(pos) = ctx.core.deck.iter().position(|y| *y == x) {
                                ctx.core.deck.remove(pos);
                            } else if let Some(pos) = ctx.core.discard.iter().position(|y| *y == x)
                            {
                                ctx.core.discard.remove(pos);
                            }
                            ctx.core.player_mut(t).hand.push(x);
                            ctx.emit(EventData::Equipped {
                                player: t,
                                card: x,
                                replaced: None,
                            });
                        }
                        let hand = ctx.core.player(t).hand.clone();
                        if !hand.is_empty() {
                            let k = ctx.core.rng.gen_range(hand.len() as u32) as usize;
                            let c = hand[k];
                            ctx.discard_from_hand(t, c);
                            ctx.log(LogEvent::CardDiscarded {
                                player: t,
                                cards: vec![c],
                            });
                        }
                    }
                }
                if let Some(c) = self.card {
                    ctx.to_discard(c);
                    ctx.core.table.retain(|x| *x != c);
                    self.card = None;
                }
                Flow::Done
            }
            // 星体改造（灵剑，规则 v198）：使用者（自目标）摸 1 + 手牌上限
            // -1 累加（其下一结束阶段重置，kernel 相位钩）；本体入弃堆。
            HostBehavior::SwdXingtigaizaoCard => {
                let p = ctx.core.player_mut(self.owner);
                p.xingtigaizao_count = p.xingtigaizao_count.saturating_add(1);
                ctx.start_draw(self.owner, 1);
                if let Some(c) = self.card {
                    ctx.to_discard(c);
                    ctx.core.table.retain(|x| *x != c);
                    self.card = None;
                }
                Flow::Done
            }
            // 时间回溯（灵剑，规则 v198）：目标获得其装备区全部牌（装备技
            // 随迁撤销）；本体入弃堆。
            HostBehavior::SwdShijianhuisuCard => {
                if let Some(t) = self.target {
                    if ctx.core.player(t).alive {
                        let es: Vec<CardId> = ctx.core.player(t).equips.iter().copied().collect();
                        for c in es {
                            if let Some(pos) =
                                ctx.core.player(t).equips.iter().position(|x| *x == c)
                            {
                                ctx.core.player_mut(t).equips.remove(pos);
                                crate::kernel::revoke_equip_skills(ctx.core, ctx.catalog, t, c);
                                ctx.core.player_mut(t).hand.push(c);
                            }
                        }
                        let hand: Vec<CardId> = ctx.core.player(t).hand.iter().copied().collect();
                        ctx.log(LogEvent::CardsDrawn {
                            player: t,
                            cards: hand,
                        });
                    }
                }
                if let Some(c) = self.card {
                    ctx.to_discard(c);
                    ctx.core.table.retain(|x| *x != c);
                    self.card = None;
                }
                Flow::Done
            }
            // 紧急冷冻（灵剑，规则 v198）：目标≠使用者且未翻面——+2 护甲、
            // 翻面；仍翻面则冻结（不能使用牌也不能成为目标，翻回解除，
            // kernel TurnOvered 钩）；本体入弃堆。
            HostBehavior::SwdJinjilengdongCard => {
                if let Some(t) = self.target {
                    if ctx.core.player(t).alive && !ctx.core.player(t).face_down {
                        // 护甲直写字段（灵雪同口径，无日志原语）
                        ctx.core.player_mut(t).hujia += 2;
                        ctx.core.player_mut(t).face_down = true;
                        ctx.emit(EventData::TurnOvered {
                            player: t,
                            face_down: true,
                        });
                        if ctx.core.player(t).face_down {
                            ctx.core.player_mut(t).jileng_active = true;
                        }
                    }
                }
                if let Some(c) = self.card {
                    ctx.to_discard(c);
                    ctx.core.table.retain(|x| *x != c);
                    self.card = None;
                }
                Flow::Done
            }
            // 圣休号角（灵剑，规则 v198）：目标获嘲讽（相邻角色只能以其为
            // 杀目标；其下一阶段开始解除，kernel 相位钩）；本体入弃堆。
            HostBehavior::SwdShengxiuhaojiaoCard => {
                if let Some(t) = self.target {
                    if ctx.core.player(t).alive {
                        ctx.core.player_mut(t).chaofeng_active = true;
                    }
                }
                if let Some(c) = self.card {
                    ctx.to_discard(c);
                    ctx.core.table.retain(|x| *x != c);
                    self.card = None;
                }
                Flow::Done
            }
            // 隐秘离场（灵剑，规则 v198）：目标≠使用者——潜行（不能成为牌
            // 的目标；当前回合结束解除，kernel 回合收束钩）；本体入弃堆。
            HostBehavior::SwdYinmilichangCard => {
                if let Some(t) = self.target {
                    if ctx.core.player(t).alive {
                        ctx.core.player_mut(t).yinmi_active = true;
                    }
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
