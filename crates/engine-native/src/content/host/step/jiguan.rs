use super::super::*;

impl HostMachine {
    pub(super) fn step_jiguan(&mut self, ctx: &mut Ctx<'_>) -> Flow {
        match self.behavior {
            HostBehavior::JiguanBingpoCard => loop {
                // 逐目标双段自选展示（逐技对照批五 v79 对齐上游 chooseCard
                // ×2）：使用者自选展示一张 → 目标自选展示一张 → 同色则
                // 使用者弃展示牌、目标失去 1 点体力（规则 v77 原语）；
                // 异色无事。展示不移动牌（仅同色弃使用者展示牌）。
                match self.stage {
                    HostStage::ComparePlayerPick => {
                        let Some(t) = self.targets.first().copied() else {
                            if let Some(c) = self.card {
                                ctx.to_discard(c);
                                ctx.core.table.retain(|x| *x != c);
                            }
                            return Flow::Done;
                        };
                        if !ctx.core.player(t).alive
                            || ctx.core.player(t).hand.is_empty()
                            || ctx.core.player(self.owner).hand.is_empty()
                        {
                            // 上游 step0：任一方无手牌该目标直接结束
                            self.targets.remove(0);
                            continue;
                        }
                        let options: Vec<ChoiceOption> = ctx
                            .core
                            .player(self.owner)
                            .hand
                            .iter()
                            .copied()
                            .map(ChoiceOption::Card)
                            .collect();
                        self.stage = HostStage::CompareTargetPick;
                        return Flow::WaitChoice {
                            observer: self.owner,
                            spec: ChoiceSpec {
                                kind: ChoiceKind::SelectCards,
                                prompt: GJ_BINGPO_SHOW_OWN,
                                options,
                                min: 1,
                                max: 1,
                                allow_cancel: false,
                            },
                        };
                    }
                    HostStage::CompareTargetPick => {
                        let Some(t) = self.targets.first().copied() else {
                            if let Some(c) = self.card {
                                ctx.to_discard(c);
                                ctx.core.table.retain(|x| *x != c);
                            }
                            return Flow::Done;
                        };
                        // 暂存使用者展示牌（cards 入快照，恢复可续）
                        self.cards = ctx
                            .answer()
                            .iter()
                            .filter_map(|o| match o {
                                ChoiceOption::Card(c) => Some(*c as u32),
                                _ => None,
                            })
                            .collect();
                        let options: Vec<ChoiceOption> = ctx
                            .core
                            .player(t)
                            .hand
                            .iter()
                            .copied()
                            .map(ChoiceOption::Card)
                            .collect();
                        self.stage = HostStage::Act;
                        return Flow::WaitChoice {
                            observer: t,
                            spec: ChoiceSpec {
                                kind: ChoiceKind::SelectCards,
                                prompt: GJ_BINGPO_SHOW_TARGET,
                                options,
                                min: 1,
                                max: 1,
                                allow_cancel: false,
                            },
                        };
                    }
                    HostStage::Act => {
                        let Some(t) = self.targets.first().copied() else {
                            if let Some(c) = self.card {
                                ctx.to_discard(c);
                                ctx.core.table.retain(|x| *x != c);
                            }
                            return Flow::Done;
                        };
                        let tc = ctx.answer().first().and_then(|o| match o {
                            ChoiceOption::Card(c) => Some(*c),
                            _ => None,
                        });
                        let own = self.cards.first().copied().map(|c| c as CardId);
                        self.cards.clear();
                        self.targets.remove(0);
                        self.stage = HostStage::ComparePlayerPick;
                        if let (Some(own), Some(tc)) = (own, tc) {
                            let color = |c: CardId| {
                                matches!(
                                    ctx.catalog.cards[c as usize].suit,
                                    crate::catalog::Suit::Heart | crate::catalog::Suit::Diamond
                                )
                            };
                            if color(own) == color(tc) {
                                ctx.core.player_mut(self.owner).hand.retain(|x| *x != own);
                                ctx.log(LogEvent::CardDiscarded {
                                    player: self.owner,
                                    cards: vec![own],
                                });
                                ctx.emit(EventData::LoseHp {
                                    player: t,
                                    amount: 1,
                                });
                            }
                        }
                        continue;
                    }
                    _ => return Flow::Done,
                }
            },
            HostBehavior::JiguanQiankunCard => {
                // 目标 + 距离 1 邻座：各弃首张手牌（无手牌弃首张装备）
                //（登记差异：随机牌改确定性首张）
                let Some(t) = self.target else {
                    return Flow::Done;
                };
                let mut victims = vec![t];
                for p in ctx.core.alive_players() {
                    if p != t
                        && p != self.owner
                        && ctx.core.computed_distance(ctx.catalog, t, p) == 1
                    {
                        victims.push(p);
                    }
                }
                for v in victims {
                    if !ctx.core.player(v).alive {
                        continue;
                    }
                    // 上游 he.randomGet()：手牌+装备合集随机一张
                    //（逐技对照批四 v78 种子 RNG 均匀抽取；changeTarget
                    // 只把 he 非空者入列——此处跳过无牌者同口径）
                    let he_len = ctx.core.player(v).hand.len() + ctx.core.player(v).equips.len();
                    if he_len == 0 {
                        continue;
                    }
                    let pick = ctx.core.rng.gen_range(he_len as u32) as usize;
                    if let Some(c) = ctx.core.player(v).hand.get(pick).copied() {
                        ctx.discard_from_hand(v, c);
                        ctx.log(LogEvent::CardDiscarded {
                            player: v,
                            cards: vec![c],
                        });
                    } else {
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
                if let Some(c) = self.card {
                    ctx.to_discard(c);
                    ctx.core.table.retain(|x| *x != c);
                }
                Flow::Done
            }
            HostBehavior::JiguanLongxuCard => match self.stage {
                HostStage::SelectCard => {
                    // 使用者自选目标装备区一张获得（上游
                    // choosePlayerCard("e", target)；逐技对照批五 v79
                    // 消除「首张」登记差异；装备区明牌，向使用者询问）
                    let Some(t) = self.target else {
                        return Flow::Done;
                    };
                    let options: Vec<ChoiceOption> = ctx
                        .core
                        .player(t)
                        .equips
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
                    self.stage = HostStage::Act;
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectCards,
                            prompt: GJ_LONGXU_PICK,
                            options,
                            min: 1,
                            max: 1,
                            allow_cancel: false,
                        },
                    }
                }
                HostStage::Act => {
                    let Some(t) = self.target else {
                        return Flow::Done;
                    };
                    let pick = ctx.answer().first().and_then(|o| match o {
                        ChoiceOption::Card(c) => Some(*c),
                        _ => None,
                    });
                    if let Some(c) = pick.filter(|c| ctx.core.player(t).equips.contains(c)) {
                        ctx.core.player_mut(t).equips.retain(|x| *x != c);
                        crate::kernel::revoke_equip_skills(ctx.core, ctx.catalog, t, c);
                        ctx.core.player_mut(self.owner).hand.push(c);
                        ctx.log(LogEvent::CardsGiven {
                            from: t,
                            to: self.owner,
                            cards: vec![c],
                        });
                    }
                    if let Some(c) = self.card {
                        ctx.to_discard(c);
                        ctx.core.table.retain(|x| *x != c);
                    }
                    Flow::Done
                }
                _ => Flow::Done,
            },
            HostBehavior::JiguanShenhuoCard => loop {
                // 目标 + 距离 1 邻座：需出闪否则受 1 点火焰伤害
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
                        let shans: Vec<ChoiceOption> = ctx
                            .core
                            .player(t)
                            .hand
                            .iter()
                            .copied()
                            .filter(|c| ctx.catalog.card_kind(*c) == CardKind::Shan)
                            .map(ChoiceOption::Card)
                            .collect();
                        if shans.is_empty() {
                            ctx.emit(EventData::Damage(crate::machine::damage_data_with(
                                Some(self.owner),
                                t,
                                1,
                                crate::event::DamageNature::Fire,
                                self.card,
                            )));
                            continue;
                        }
                        let mut options = vec![ChoiceOption::Pass];
                        options.extend(shans);
                        self.stage = HostStage::TargetChoice;
                        return Flow::WaitChoice {
                            observer: t,
                            spec: ChoiceSpec {
                                kind: ChoiceKind::RespondCard,
                                prompt: "aoe_response",
                                options,
                                min: 1,
                                max: 1,
                                allow_cancel: false,
                            },
                        };
                    }
                    HostStage::TargetChoice => {
                        self.stage = HostStage::Act;
                        let answered = ctx.answer().first().copied();
                        let responder = self.target;
                        if let (Some(ChoiceOption::Card(c)), Some(t)) = (answered, responder) {
                            ctx.discard_from_hand(t, c);
                            ctx.emit(EventData::CardResponded { card: c, player: t });
                        } else if let Some(t) = responder {
                            ctx.emit(EventData::Damage(crate::machine::damage_data_with(
                                Some(self.owner),
                                t,
                                1,
                                crate::event::DamageNature::Fire,
                                self.card,
                            )));
                        }
                        continue;
                    }
                    _ => return Flow::Done,
                }
            },
            HostBehavior::JiguanShatangCard => match self.stage {
                HostStage::Act => {
                    // 目标受 1 点火焰伤害——emit 后 Yield，待伤害帧
                    // 提交后再给护甲（否则新护甲吸收本次伤害）
                    let Some(t) = self.target else {
                        return Flow::Done;
                    };
                    if ctx.core.player(t).alive {
                        ctx.emit(EventData::Damage(crate::machine::damage_data_with(
                            Some(self.owner),
                            t,
                            1,
                            crate::event::DamageNature::Fire,
                            self.card,
                        )));
                        self.stage = HostStage::SelectNature;
                        return Flow::Yield;
                    }
                    if let Some(c) = self.card {
                        ctx.to_discard(c);
                        ctx.core.table.retain(|x| *x != c);
                    }
                    Flow::Done
                }
                HostStage::SelectNature => {
                    // 伤害已提交：获得 1 点护甲并收尾
                    let Some(t) = self.target else {
                        return Flow::Done;
                    };
                    if ctx.core.player(t).alive {
                        ctx.change_hujia(t, 1);
                    }
                    if let Some(c) = self.card {
                        ctx.to_discard(c);
                        ctx.core.table.retain(|x| *x != c);
                    }
                    Flow::Done
                }
                _ => Flow::Done,
            },
            HostBehavior::JiguanHeilongCard => {
                // 获得 1 点护甲（登记差异：防御距离 +1 临时技暂缓）
                ctx.change_hujia(self.owner, 1);
                if let Some(c) = self.card {
                    ctx.to_discard(c);
                    ctx.core.table.retain(|x| *x != c);
                }
                Flow::Done
            }
            HostBehavior::JiguanLiutougeCard => {
                // 自己 → 摸 1（增益简化）；其他角色 → 弃首张手牌
                //（减益简化；登记差异：上游随机效果池）
                let Some(t) = self.target else {
                    return Flow::Done;
                };
                if ctx.core.player(t).alive {
                    if t == self.owner {
                        ctx.draw_cards(t, 1);
                    } else if let Some(c) = ctx.core.player(t).hand.first().copied() {
                        ctx.discard_from_hand(t, c);
                        ctx.log(LogEvent::CardDiscarded {
                            player: t,
                            cards: vec![c],
                        });
                    }
                }
                if let Some(c) = self.card {
                    ctx.to_discard(c);
                    ctx.core.table.retain(|x| *x != c);
                }
                Flow::Done
            }
            HostBehavior::JiguanDinvCard => match self.stage {
                HostStage::Act => {
                    // 回复 1 点体力，然后可弃任意张手牌摸等量（可取消）
                    ctx.recover(self.owner, 1, None);
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
                    let n = options.len();
                    self.stage = HostStage::SelectCard;
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectCards,
                            prompt: GJ_DINVXUANSHUANG_RIDER,
                            options,
                            min: 0,
                            max: n,
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
                    if !picked.is_empty() {
                        for c in &picked {
                            ctx.core.player_mut(self.owner).hand.retain(|x| *x != *c);
                        }
                        ctx.log(LogEvent::CardDiscarded {
                            player: self.owner,
                            cards: picked.clone(),
                        });
                        ctx.draw_cards(self.owner, picked.len());
                    }
                    if let Some(c) = self.card {
                        ctx.to_discard(c);
                        ctx.core.table.retain(|x| *x != c);
                    }
                    Flow::Done
                }
                _ => Flow::Done,
            },
            _ => unreachable!("behavior routed to the wrong step module"),
        }
    }
}
