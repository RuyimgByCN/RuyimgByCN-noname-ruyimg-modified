use super::super::*;

impl HostMachine {
    pub(super) fn step_hearth(&mut self, ctx: &mut Ctx<'_>) -> Flow {
        match self.behavior {
            HostBehavior::HearthDunpaiCard => match self.stage {
                HostStage::Act => {
                    // +1 护甲、摸 1——R05-C2：摸 1 走 R 帧；「弃首张手牌」
                    // 读摸后手牌（空手时同步会弃掉刚摸的牌）→ 续行段。
                    ctx.change_hujia(self.owner, 1);
                    ctx.start_draw(self.owner, 1);
                    self.stage = HostStage::DrawSettle;
                    Flow::Yield
                }
                // R05-C2 续行位：盾牌摸 1 交付后弃首张+本体入弃堆。
                HostStage::DrawSettle => {
                    let options: Vec<ChoiceOption> = ctx
                        .core
                        .player(self.owner)
                        .hand
                        .iter()
                        .copied()
                        .map(ChoiceOption::Card)
                        .collect();
                    if options.is_empty() {
                        if let Some(c) = self.card {
                            ctx.to_discard(c);
                            ctx.core.table.retain(|x| *x != c);
                        }
                        return Flow::Done;
                    }
                    // 弃 1（登记差异：确定性首张）
                    let c0 = match options.first() {
                        Some(ChoiceOption::Card(c)) => *c,
                        _ => return Flow::Done,
                    };
                    ctx.core.player_mut(self.owner).hand.retain(|x| *x != c0);
                    ctx.log(LogEvent::CardDiscarded {
                        player: self.owner,
                        cards: vec![c0],
                    });
                    if let Some(c) = self.card {
                        ctx.to_discard(c);
                        ctx.core.table.retain(|x| *x != c);
                    }
                    Flow::Done
                }
                _ => Flow::Done,
            },
            HostBehavior::HearthZhiliaoboCard => {
                // 判定红色 → 回复 1；否则 +1 护甲
                let Some(t) = self.target else {
                    return Flow::Done;
                };
                let judged = ctx.draw_judge_card();
                let red = matches!(
                    ctx.catalog.cards[judged as usize].suit,
                    crate::catalog::Suit::Heart | crate::catalog::Suit::Diamond
                );
                ctx.to_discard(judged);
                if red {
                    ctx.recover(t, 1, None);
                } else {
                    ctx.change_hujia(t, 1);
                }
                Flow::Done
            }
            HostBehavior::HearthYuansuCard => loop {
                // 逐目标：可弃 1–2 张（可取消）→ 每少弃一张受 1 雷伤
                match self.stage {
                    HostStage::Act => {
                        let Some(t) = self.targets.first().copied() else {
                            if let Some(c) = self.card {
                                ctx.to_discard(c);
                                ctx.core.table.retain(|x| *x != c);
                            }
                            return Flow::Done;
                        };
                        self.targets.remove(0);
                        if !ctx.core.player(t).alive {
                            continue;
                        }
                        self.target = Some(t);
                        let hand: Vec<ChoiceOption> = ctx
                            .core
                            .player(t)
                            .hand
                            .iter()
                            .chain(ctx.core.player(t).equips.iter())
                            .copied()
                            .map(ChoiceOption::Card)
                            .collect();
                        if hand.is_empty() {
                            ctx.emit(EventData::Damage(crate::machine::damage_data_with(
                                Some(self.owner),
                                t,
                                2,
                                crate::event::DamageNature::Thunder,
                                self.card,
                            )));
                            continue;
                        }
                        let n = hand.len().min(2);
                        self.stage = HostStage::TargetChoice;
                        return Flow::WaitChoice {
                            observer: t,
                            spec: ChoiceSpec {
                                kind: ChoiceKind::SelectCards,
                                prompt: HH_YUANSU_DISCARD,
                                options: hand,
                                min: 1,
                                max: n,
                                allow_cancel: true,
                            },
                        };
                    }
                    HostStage::TargetChoice => {
                        self.stage = HostStage::Act;
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
                        for c in &picked {
                            ctx.core.player_mut(t).hand.retain(|x| *x != *c);
                            ctx.core.player_mut(t).equips.retain(|x| *x != *c);
                            ctx.to_discard(*c);
                        }
                        if !picked.is_empty() {
                            ctx.log(LogEvent::CardDiscarded {
                                player: t,
                                cards: picked.clone(),
                            });
                        }
                        let dmg = 2 - picked.len() as u32;
                        if dmg > 0 {
                            ctx.emit(EventData::Damage(crate::machine::damage_data_with(
                                Some(self.owner),
                                t,
                                dmg as i32,
                                crate::event::DamageNature::Thunder,
                                self.card,
                            )));
                        }
                        continue;
                    }
                    _ => return Flow::Done,
                }
            },
            HostBehavior::HearthXingjieCard => {
                // 体力上限 +1、回复 1、弃全部手牌
                ctx.core.player_mut(self.owner).max_hp += 1;
                ctx.recover(self.owner, 1, None);
                let hand: Vec<CardId> = ctx.core.player(self.owner).hand.clone();
                for c in &hand {
                    ctx.core.player_mut(self.owner).hand.retain(|x| x != c);
                }
                if !hand.is_empty() {
                    ctx.log(LogEvent::CardDiscarded {
                        player: self.owner,
                        cards: hand,
                    });
                }
                if let Some(c) = self.card {
                    ctx.to_discard(c);
                    ctx.core.table.retain(|x| *x != c);
                }
                Flow::Done
            }
            HostBehavior::HearthChuansongCard => match self.stage {
                HostStage::Act => {
                    // 获得牌堆顶一张牌（登记差异：立即使用链暂缓）——
                    // R05-C2：摸 1 走 R 帧；本体入弃堆在续行段（同步序＝
                    // 摸先于本体入弃堆，洗牌边界下序敏感）。
                    ctx.start_draw(self.owner, 1);
                    self.stage = HostStage::DrawSettle;
                    Flow::Yield
                }
                // R05-C2 续行位：传送摸 1 交付后本体入弃堆。
                HostStage::DrawSettle => {
                    if let Some(c) = self.card {
                        ctx.to_discard(c);
                        ctx.core.table.retain(|x| *x != c);
                    }
                    Flow::Done
                }
                _ => Flow::Done,
            },
            HostBehavior::HearthTansheCard => {
                // 座位序链式弃首张手牌，共「存活数取奇」次
                //（登记差异：随机敌随机牌改确定性座位链）
                let mut current = self.owner;
                let mut times = {
                    let alive = ctx.core.alive_count();
                    if alive % 2 == 0 {
                        alive - 1
                    } else {
                        alive
                    }
                };
                while times > 0 {
                    let next = ctx.core.seat_order_from(current).into_iter().find(|p| {
                        *p != current
                            && ctx.core.player(*p).alive
                            && !ctx.core.player(*p).hand.is_empty()
                    });
                    match next {
                        Some(v) => {
                            let c = ctx.core.player(v).hand.first().copied();
                            if let Some(c) = c {
                                ctx.discard_from_hand(v, c);
                                ctx.log(LogEvent::CardDiscarded {
                                    player: v,
                                    cards: vec![c],
                                });
                            }
                            current = v;
                            times -= 1;
                        }
                        None => break,
                    }
                }
                if let Some(c) = self.card {
                    ctx.to_discard(c);
                    ctx.core.table.retain(|x| *x != c);
                }
                Flow::Done
            }
            HostBehavior::HearthShijieshuCard => {
                // 牌堆自顶搜索两张不同槽位的装备，目标依次穿戴
                //（同槽旧装备撤销技能后置入弃牌堆）
                let Some(t) = self.target else {
                    return Flow::Done;
                };
                let slot_of = |c: CardId| ctx.catalog.cards[c as usize].kind.equip_slot();
                let mut picked: Vec<CardId> = Vec::new();
                let mut seen_slots: Vec<_> = Vec::new();
                // 牌堆自顶（向量尾）向底扫描
                let mut i = ctx.core.deck.len();
                while i > 0 && picked.len() < 2 {
                    i -= 1;
                    let c = ctx.core.deck[i];
                    let Some(slot) = slot_of(c) else { continue };
                    if seen_slots.contains(&slot) {
                        continue;
                    }
                    seen_slots.push(slot);
                    picked.push(c);
                    ctx.core.deck.remove(i);
                }
                for c in picked {
                    let slot = slot_of(c);
                    // 同槽旧装备替换
                    let replaced = ctx
                        .core
                        .player(t)
                        .equips
                        .iter()
                        .copied()
                        .find(|e| ctx.catalog.cards[*e as usize].kind.equip_slot() == slot);
                    if let Some(old) = replaced {
                        ctx.core.player_mut(t).equips.retain(|x| *x != old);
                        crate::kernel::revoke_equip_skills(ctx.core, ctx.catalog, t, old);
                        ctx.to_discard(old);
                    }
                    ctx.core.player_mut(t).equips.push(c);
                    crate::kernel::grant_equip_skills(ctx, t, c);
                    ctx.log(LogEvent::Equipped {
                        player: t,
                        card: c,
                        replaced,
                    });
                }
                if let Some(c) = self.card {
                    ctx.to_discard(c);
                    ctx.core.table.retain(|x| *x != c);
                }
                Flow::Done
            }
            _ => unreachable!("behavior routed to the wrong step module"),
        }
    }
}
