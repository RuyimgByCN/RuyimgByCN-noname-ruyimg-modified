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
                                ctx.discard_from_hand(self.owner, own);
                                ctx.emit(EventData::LoseHp {
                                    player: t,
                                    amount: 1,
                                });
                                if let Some(c) = self.card {
                                    if ctx.core.table.contains(&c) {
                                        ctx.to_discard(c);
                                        ctx.core.table.retain(|x| *x != c);
                                    }
                                }
                                return Flow::Done;
                            }
                        }
                        continue;
                    }
                    _ => return Flow::Done,
                }
            },
            HostBehavior::JiguanQiankunCard => {
                // 目标 + 纯距 1 邻座（含使用者，card/gujian/card.js:658）：各随机弃一张 he 牌
                let Some(t) = self.target else {
                    return Flow::Done;
                };
                let mut victims = vec![t];
                for p in ctx.core.seat_order_from(t) {
                    if p != t
                        && ctx.core.player(p).alive
                        && ctx.core.seat_distance_alive(t, p) == 1
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
                        let slot = ctx.catalog.card_slot_of(ctx.core, c);
                        let replaced = ctx
                            .core
                            .player(self.owner)
                            .equips
                            .iter()
                            .copied()
                            .find(|e| ctx.catalog.card_slot_of(ctx.core, *e) == slot);
                        if let Some(old) = replaced {
                            ctx.core.player_mut(self.owner).equips.retain(|x| *x != old);
                            crate::kernel::revoke_equip_skills(
                                ctx.core,
                                ctx.catalog,
                                self.owner,
                                old,
                            );
                            ctx.to_discard(old);
                        }
                        ctx.core.player_mut(self.owner).equips.push(c);
                        crate::kernel::grant_equip_skills(ctx, self.owner, c);
                        ctx.log(LogEvent::Equipped {
                            player: self.owner,
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
                // 获得 1 点护甲 + 临时防御距离 +1（至自己下回合开始）
                ctx.change_hujia(self.owner, 1);
                ctx.core
                    .vm_aux
                    .flags
                    .insert((self.owner, "u05_mod:gj_heilonglinpian".to_string()), 1);
                if let Some(c) = self.card {
                    ctx.to_discard(c);
                    ctx.core.table.retain(|x| *x != c);
                }
                Flow::Done
            }
            HostBehavior::JiguanLiutougeCard => {
                let Some(t) = self.target else {
                    return Flow::Done;
                };
                if ctx.core.player(t).alive {
                    if t != self.owner && !ctx.core.is_friend_of(ctx.catalog, self.owner, t) {
                        super::u05_b2_gujian::helpers::apply_get_debuff(ctx, t, false);
                    } else {
                        super::u05_b2_gujian::helpers::apply_get_buff(ctx, t);
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
                    // 目标（濒死救回者或主动自用者）回复 1 点体力，然后可弃任意张 he 牌摸等量
                    let tgt = self.target.unwrap_or(self.owner);
                    let already_recovered = ctx
                        .core
                        .vm_aux
                        .flags
                        .remove(&(self.owner, "dinv_already_recovered".to_string()))
                        .is_some();
                    if !already_recovered {
                        ctx.recover(tgt, 1, None);
                    }
                    if !ctx.core.player(tgt).alive {
                        if let Some(c) = self.card {
                            if ctx.core.table.contains(&c) {
                                ctx.to_discard(c);
                                ctx.core.table.retain(|x| *x != c);
                            }
                        }
                        return Flow::Done;
                    }
                    let mut options: Vec<ChoiceOption> = ctx
                        .core
                        .player(tgt)
                        .hand
                        .iter()
                        .copied()
                        .map(ChoiceOption::Card)
                        .collect();
                    options.extend(
                        ctx.core
                            .player(tgt)
                            .equips
                            .iter()
                            .copied()
                            .map(ChoiceOption::Card),
                    );
                    if options.is_empty() {
                        if let Some(c) = self.card {
                            if ctx.core.table.contains(&c) {
                                ctx.to_discard(c);
                                ctx.core.table.retain(|x| *x != c);
                            }
                        }
                        return Flow::Done;
                    }
                    let n = options.len();
                    self.stage = HostStage::SelectCard;
                    Flow::WaitChoice {
                        observer: tgt,
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
                    let tgt = self.target.unwrap_or(self.owner);
                    let picked: Vec<CardId> = ctx
                        .answer()
                        .iter()
                        .filter_map(|o| match o {
                            ChoiceOption::Card(c) => Some(*c),
                            _ => None,
                        })
                        .collect();
                    if !picked.is_empty() {
                        let mut lost = Vec::new();
                        let mut origins = Vec::new();
                        for &c in &picked {
                            if ctx.core.player(tgt).hand.contains(&c) {
                                ctx.take_hand_card(tgt, c);
                                ctx.to_discard(c);
                                lost.push(c);
                                origins.push(crate::event::LostOrigin::Hand);
                            } else if ctx.core.player(tgt).equips.contains(&c) {
                                ctx.core.player_mut(tgt).equips.retain(|x| *x != c);
                                ctx.core.declared_slots.remove(&c);
                                crate::kernel::revoke_equip_skills(ctx.core, ctx.catalog, tgt, c);
                                ctx.to_discard(c);
                                lost.push(c);
                                origins.push(crate::event::LostOrigin::Equip);
                            }
                        }
                        if !lost.is_empty() {
                            ctx.note_cards_lost(tgt, &lost);
                            ctx.log(LogEvent::CardDiscarded {
                                player: tgt,
                                cards: lost.clone(),
                            });
                            let draw_cnt = lost.len() as u32;
                            ctx.start_draw(tgt, draw_cnt);
                            ctx.emit(EventData::CardsLost {
                                player: tgt,
                                cards: lost,
                                origins,
                            });
                        }
                    }
                    if let Some(c) = self.card {
                        if ctx.core.table.contains(&c) {
                            ctx.to_discard(c);
                            ctx.core.table.retain(|x| *x != c);
                        }
                    }
                    Flow::Done
                }
                _ => Flow::Done,
            },
            _ => unreachable!("behavior routed to the wrong step module"),
        }
    }
}
