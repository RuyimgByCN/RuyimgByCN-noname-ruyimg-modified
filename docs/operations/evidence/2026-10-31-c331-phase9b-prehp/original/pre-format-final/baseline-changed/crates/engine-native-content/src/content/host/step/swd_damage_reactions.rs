use super::super::*;

impl HostMachine {
    pub(super) fn step_swd_damage_reactions(&mut self, ctx: &mut Ctx<'_>) -> Flow {
        match self.behavior {
            // 隳延：SelectCard 弃一张牌 → 弃置对方全部装备（随 revoke）
            HostBehavior::SwdSuiyan => match self.stage {
                HostStage::Act => {
                    let Some(d) = ctx
                        .res
                        .frame(ctx.current_frame_id())
                        .map(|f| &f.data)
                        .and_then(|data| match data {
                            crate::event::EventData::Damage(d) => Some(d.target),
                            _ => None,
                        })
                    else {
                        return Flow::Done;
                    };
                    self.target = Some(d);
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
                            prompt: "suiyan_discard",
                            options: pool.iter().copied().map(ChoiceOption::Card).collect(),
                            min: 0,
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
                    let t = match self.target {
                        Some(t) => t,
                        None => return Flow::Done,
                    };
                    let equips: Vec<CardId> = ctx.core.player(t).equips.iter().copied().collect();
                    for c in equips {
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
            // 破浪：SelectCards 选对方装备一张（可取消=放弃；公开信息自选
            // 面保留）→ 弃置
            HostBehavior::SwdPolang => match self.stage {
                HostStage::Act => {
                    let Some(d) = ctx
                        .res
                        .frame(ctx.current_frame_id())
                        .map(|f| &f.data)
                        .and_then(|data| match data {
                            crate::event::EventData::Damage(d) => Some(d.target),
                            _ => None,
                        })
                    else {
                        return Flow::Done;
                    };
                    self.target = Some(d);
                    let options: Vec<ChoiceOption> = ctx
                        .core
                        .player(d)
                        .equips
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
                            prompt: "polang_discard_equip",
                            options,
                            min: 0,
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
                    if ctx.core.player(t).equips.contains(&picked) {
                        ctx.core.player_mut(t).equips.retain(|x| *x != picked);
                        if ctx.catalog.card_kind(picked).is_equip() {
                            crate::kernel::revoke_equip_skills(ctx.core, ctx.catalog, t, picked);
                        }
                        ctx.to_discard(picked);
                        ctx.log(LogEvent::CardDiscarded {
                            player: t,
                            cards: vec![picked],
                        });
                    }
                    Flow::Done
                }
                _ => Flow::Done,
            },
            // 天火·秦壬面：判定区有牌的其他角色各——弃判定牌+受等量火伤
            //（nosource 登记差异：原生 Damage 需 source，改 owner 登记差异）
            HostBehavior::SwdTianhuo => {
                let victims: Vec<PlayerId> = ctx
                    .core
                    .seat_order_from(self.owner)
                    .into_iter()
                    .filter(|p| *p != self.owner && !ctx.core.player(*p).judge.is_empty())
                    .collect();
                for v in victims {
                    let n = ctx.core.player(v).judge.len() as u32;
                    let judge_cards: Vec<CardId> =
                        ctx.core.player(v).judge.iter().copied().collect();
                    for c in judge_cards {
                        ctx.core.player_mut(v).judge.retain(|x| *x != c);
                        ctx.to_discard(c);
                        ctx.log(LogEvent::CardDiscarded {
                            player: v,
                            cards: vec![c],
                        });
                    }
                    ctx.emit(EventData::Damage(crate::machine::damage_data_with(
                        Some(self.owner),
                        v,
                        n as i32,
                        crate::event::DamageNature::Fire,
                        None,
                    )));
                }
                Flow::Done
            }
            // 化金触发面：伤害前置 +1；准备阶段开始移除实例
            HostBehavior::SwdHuajin2 => {
                let frame = ctx.res.frame(ctx.current_frame_id()).map(|f| &f.data);
                match frame {
                    Some(EventData::Damage(d)) if d.source == Some(self.owner) && !d.prevented => {
                        if !ctx.modify_damage_amount(1) { return Flow::Done; }
                        Flow::Done
                    }
                    Some(EventData::Phase {
                        player,
                        phase: crate::event::Phase::Judge,
                        begin: true,
                    }) if *player == self.owner => {
                        if let Some(def) = ctx.catalog.skill_by_name(SWD_HUAJIN2_KEY) {
                            ctx.core.skills.remove(&(self.owner, def));
                        }
                        Flow::Done
                    }
                    _ => Flow::Done,
                }
            }
            // 因果：确认 → 弃一张 he → DamageBefore 帧 source/target 对调
            //（unique 一次移除实例）
            HostBehavior::SwdYinguo => match self.stage {
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
                            prompt: "yinguo_discard",
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
                    // 帧 source/target 对调（redirect_chain 不动——上游仅换
                    // 两字段）
                    let frame = ctx.res.frame_mut(ctx.current_frame_id()).expect("frame");
                    if let crate::event::EventData::Damage(d) = &mut frame.data {
                        let t = d.target;
                        d.target = d.source.expect("filter guarantees source");
                        d.source = Some(t);
                    }
                    if let Some(d) = ctx.catalog.skill_by_name("yinguo") {
                        ctx.core.skills.remove(&(self.owner, d));
                    }
                    Flow::Done
                }
                _ => Flow::Done,
            },
            // 破阵：Act 读差值分流——多：弃差值张→来源 1 伤；少：确认
            // →弃来源差值张（确定性首 N 张）
            HostBehavior::SwdPozhen => match self.stage {
                HostStage::Act => {
                    let Some(d) = ctx
                        .res
                        .frame(ctx.current_frame_id())
                        .map(|f| &f.data)
                        .and_then(|data| match data {
                            crate::event::EventData::Damage(d) => Some((d.target, d.source)),
                            _ => None,
                        })
                    else {
                        return Flow::Done;
                    };
                    let Some(src) = d.1 else {
                        return Flow::Done;
                    };
                    self.target = Some(src);
                    let my = ctx.core.player(self.owner).hand.len() as i32;
                    let their = ctx.core.player(src).hand.len() as i32;
                    let diff = my - their;
                    if diff > 0 {
                        let n = diff as usize;
                        let hand: Vec<ChoiceOption> = ctx
                            .core
                            .player(self.owner)
                            .hand
                            .iter()
                            .copied()
                            .map(ChoiceOption::Card)
                            .collect();
                        if hand.len() < n {
                            return Flow::Done;
                        }
                        self.cards = vec![0; n];
                        self.stage = HostStage::SelectCard;
                        Flow::WaitChoice {
                            observer: self.owner,
                            spec: ChoiceSpec {
                                kind: ChoiceKind::SelectCards,
                                prompt: "pozhen_discard",
                                options: hand,
                                min: n,
                                max: n,
                                allow_cancel: true,
                            },
                        }
                    } else if diff < 0 {
                        let n = (-diff) as usize;
                        if ctx.core.player(src).hand.len() < n {
                            return Flow::Done;
                        }
                        self.cards = vec![0; n];
                        self.stage = HostStage::TargetChoice;
                        Flow::WaitChoice {
                            observer: self.owner,
                            spec: ChoiceSpec {
                                kind: ChoiceKind::Confirm,
                                prompt: "pozhen_discard_source",
                                options: vec![ChoiceOption::Confirm],
                                min: 0,
                                max: 1,
                                allow_cancel: true,
                            },
                        }
                    } else {
                        Flow::Done
                    }
                }
                HostStage::SelectCard => {
                    let picks: Vec<CardId> = ctx
                        .answer()
                        .iter()
                        .filter_map(|o| match o {
                            ChoiceOption::Card(c) => Some(*c),
                            _ => None,
                        })
                        .collect();
                    let n = self.cards.len();
                    if picks.len() != n {
                        return Flow::Done;
                    }
                    for c in picks {
                        if ctx.core.player(self.owner).hand.contains(&c) {
                            ctx.discard_from_hand(self.owner, c);
                        }
                    }
                    let Some(t) = self.target else {
                        return Flow::Done;
                    };
                    if ctx.core.player(t).alive {
                        ctx.emit(EventData::Damage(crate::machine::damage_data_with(
                            Some(self.owner),
                            t,
                            1,
                            crate::event::DamageNature::Normal,
                            None,
                        )));
                    }
                    self.cards.clear();
                    self.target = None;
                    Flow::Done
                }
                HostStage::TargetChoice => {
                    // 少分支确认消费：弃来源差值张（确定性首 N）
                    let confirmed = matches!(ctx.answer().first(), Some(ChoiceOption::Confirm));
                    let n = self.cards.len();
                    let Some(t) = self.target else {
                        return Flow::Done;
                    };
                    if confirmed && ctx.core.player(t).alive {
                        let hand: Vec<CardId> = ctx.core.player(t).hand.iter().copied().collect();
                        for c in hand.into_iter().take(n) {
                            ctx.discard_from_hand(t, c);
                        }
                    }
                    self.cards.clear();
                    self.target = None;
                    Flow::Done
                }
                _ => Flow::Done,
            },
            // 黑火：SelectCard 弃一张装备牌（he）→ 摸等同弃前手牌数的牌
            // → 若手牌 ≥8 自受 3 火伤
            HostBehavior::SwdHeihuo => match self.stage {
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
                            prompt: "heihuo_discard",
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
                    let draw_n = ctx.core.player(self.owner).hand.len();
                    if draw_n > 0 {
                        ctx.draw_cards(self.owner, draw_n);
                    }
                    if ctx.core.player(self.owner).hand.len() >= 8 {
                        ctx.emit(EventData::Damage(crate::machine::damage_data_with(
                            Some(self.owner),
                            self.owner,
                            3,
                            crate::event::DamageNature::Fire,
                            None,
                        )));
                    }
                    Flow::Done
                }
                _ => Flow::Done,
            },
            // 烈刃：与受害者拼点——CompareTargetPick 目标选牌 →
            // ComparePlayerPick 己方选牌 → 赢者获得对方拼点牌+随机一张 he
            HostBehavior::SwdLieren => match self.stage {
                HostStage::Act => {
                    let victim = ctx
                        .res
                        .frame(ctx.current_frame_id())
                        .map(|f| &f.data)
                        .and_then(|data| match data {
                            crate::event::EventData::Damage(d) => Some(d.target),
                            _ => None,
                        });
                    let Some(t) = victim else {
                        return Flow::Done;
                    };
                    if !ctx.core.player(t).alive
                        || ctx.core.player(self.owner).hand.is_empty()
                        || ctx.core.player(t).hand.is_empty()
                    {
                        return Flow::Done;
                    }
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
                    Flow::WaitChoice {
                        observer: t,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectCards,
                            prompt: "lieren_target_pick",
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
                            prompt: "lieren_player_pick",
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
                    if ctx.core.player(self.owner).hand.contains(&p_card) {
                        ctx.discard_from_hand(self.owner, p_card);
                    }
                    if ctx.core.player(t).hand.contains(&t_card) {
                        ctx.discard_from_hand(t, t_card);
                    }
                    let pr = ctx.catalog.cards[p_card as usize].rank;
                    let tr = ctx.catalog.cards[t_card as usize].rank;
                    if pr > tr {
                        // 赢：获得目标一张 he（手牌优先；登记差异：上游
                        // gainPlayerCard 自选，原生确定性首张）
                        let grab = ctx
                            .core
                            .player(t)
                            .hand
                            .first()
                            .copied()
                            .map(|c| (t, c, true))
                            .or_else(|| {
                                ctx.core
                                    .player(t)
                                    .equips
                                    .first()
                                    .copied()
                                    .map(|c| (t, c, false))
                            });
                        if let Some((from, c, in_hand)) = grab {
                            if in_hand {
                                ctx.core.player_mut(from).hand.retain(|x| *x != c);
                            } else {
                                ctx.core.player_mut(from).equips.retain(|x| *x != c);
                                if ctx.catalog.card_kind(c).is_equip() {
                                    crate::kernel::revoke_equip_skills(
                                        ctx.core,
                                        ctx.catalog,
                                        from,
                                        c,
                                    );
                                }
                            }
                            ctx.core.player_mut(self.owner).hand.push(c);
                            ctx.log(LogEvent::CardsGiven {
                                from,
                                to: self.owner,
                                cards: vec![c],
                            });
                        }
                    }
                    Flow::Done
                }
                _ => Flow::Done,
            },
            // 麻醉触发面：伤害前置——off_court 黑牌返还（登记差异：上游
            // 伤害后归还，原生伤前归还）并令本次伤害 -1
            HostBehavior::SwdLmazui2 => {
                let black = ctx
                    .core
                    .player(self.owner)
                    .off_court
                    .iter()
                    .copied()
                    .find(|c| {
                        matches!(
                            ctx.catalog.cards[*c as usize].suit,
                            crate::catalog::Suit::Spade | crate::catalog::Suit::Club
                        )
                    });
                if let Some(c) = black {
                    ctx.core
                        .player_mut(self.owner)
                        .off_court
                        .retain(|x| *x != c);
                    ctx.core.player_mut(self.owner).hand.push(c);
                }
                let frame = ctx.res.frame_mut(ctx.current_frame_id()).expect("frame");
                if let crate::event::EventData::Damage(d) = &mut frame.data {
                    d.amount = d.amount.saturating_sub(1);
                }
                if let Some(def) = ctx.catalog.skill_by_name(SWD_LMAZUI2_KEY) {
                    ctx.core.skills.remove(&(self.owner, def));
                }
                Flow::Done
            }
            // 血裔·Miles 面：锁定——伤害前置取消+失去 1 点体力（无条件；
            // Boss 绝情同窗）
            HostBehavior::SwdMilesXueyi => {
                let frame = ctx.res.frame_mut(ctx.current_frame_id()).expect("frame");
                if let crate::event::EventData::Damage(d) = &mut frame.data {
                    d.prevented = true;
                }
                ctx.emit(EventData::LoseHp {
                    player: self.owner,
                    amount: 1,
                });
                Flow::Done
            }
            // 观虎：确认后——弃目标一张手牌+一张装备（确定性首张，登记
            // 差异上游自选 discardPlayerCard 可选面）
            HostBehavior::SwdGuanhu => match self.stage {
                HostStage::Act => {
                    let Some(d) = ctx
                        .res
                        .frame(ctx.current_frame_id())
                        .map(|f| &f.data)
                        .and_then(|data| match data {
                            crate::event::EventData::Damage(d) => Some(d.target),
                            _ => None,
                        })
                    else {
                        return Flow::Done;
                    };
                    self.target = Some(d);
                    // 自选窗：目标手牌+装备合集任选（min 0 可取消=放弃）
                    let mut pool: Vec<CardId> = ctx.core.player(d).hand.iter().copied().collect();
                    pool.extend(ctx.core.player(d).equips.iter().copied());
                    if pool.is_empty() {
                        return Flow::Done;
                    }
                    self.stage = HostStage::SelectCard;
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectCards,
                            prompt: "guanhu_discard",
                            options: pool.iter().copied().map(ChoiceOption::Card).collect(),
                            min: 0,
                            max: 2,
                            allow_cancel: true,
                        },
                    }
                }
                HostStage::SelectCard => {
                    let picks: Vec<CardId> = ctx
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
                    for c in picks {
                        let in_hand = ctx.core.player(t).hand.contains(&c);
                        if in_hand {
                            ctx.take_hand_card(t, c);
                        } else if ctx.core.player(t).equips.contains(&c) {
                            ctx.core.player_mut(t).equips.retain(|x| *x != c);
                            if ctx.catalog.card_kind(c).is_equip() {
                                crate::kernel::revoke_equip_skills(ctx.core, ctx.catalog, t, c);
                            }
                        } else {
                            continue;
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
            // 乖戾：Before 帧 +1（缚斧同款直写）；After 帧弃两张手牌
            //（手牌不足弃全部）
            HostBehavior::SwdGuaili => {
                let frame = ctx.res.frame(ctx.current_frame_id()).map(|f| &f.data);
                match frame {
                    Some(crate::event::EventData::Damage(d))
                        if !d.prevented && d.source == Some(self.owner) =>
                    {
                        // Before/After 同载荷同 kind——按帧 stage 区分：
                        // BeforeTriggers 面直写 +1；AfterTriggers 面弃两张
                        let stage = ctx.res.frame(ctx.current_frame_id()).expect("frame").stage;
                        if stage == crate::event::Stage::AfterTriggers {
                            let hand: Vec<CardId> =
                                ctx.core.player(self.owner).hand.iter().copied().collect();
                            let n = hand.len().min(2);
                            for c in hand.into_iter().take(n) {
                                ctx.discard_from_hand(self.owner, c);
                            }
                            Flow::Done
                        } else {
                            let frame = ctx.res.frame_mut(ctx.current_frame_id()).expect("frame");
                            if let crate::event::EventData::Damage(d) = &mut frame.data {
                                if let Some(amount) = d.amount.checked_add(1) { d.amount = amount; } else { ctx.fail_damage_amount(); return Flow::Done; }
                            }
                            Flow::Done
                        }
                    }
                    _ => Flow::Done,
                }
            }
            // 虚影：锁定——伤害前置取消 + 手牌非空则失去 1 点体力（Boss
            // 绝情同窗：帧 cancelled 写回+LoseHp 发射）
            HostBehavior::SwdXuying => {
                let frame = ctx.res.frame_mut(ctx.current_frame_id()).expect("frame");
                if let crate::event::EventData::Damage(d) = &mut frame.data {
                    d.prevented = true;
                }
                if !ctx.core.player(self.owner).hand.is_empty() {
                    ctx.emit(EventData::LoseHp {
                        player: self.owner,
                        amount: 1,
                    });
                }
                Flow::Done
            }
            // 朱羽：Act 问选红牌 → SelectCard 弃置+伤害 +1 且改火属性
            //（取消=伤害照常）
            HostBehavior::SwdZhuyu => match self.stage {
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
                            prompt: "zhuyu_discard",
                            options,
                            min: 0,
                            max: 1,
                            allow_cancel: true,
                        },
                    }
                }
                HostStage::SelectCard => {
                    let picked: Option<CardId> = ctx.answer().iter().find_map(|o| match o {
                        ChoiceOption::Card(c) => Some(*c),
                        _ => None,
                    });
                    let Some(discarded) = picked else {
                        return Flow::Done;
                    };
                    if ctx.core.player(self.owner).hand.contains(&discarded) {
                        ctx.discard_from_hand(self.owner, discarded);
                    } else {
                        ctx.core
                            .player_mut(self.owner)
                            .equips
                            .retain(|x| *x != discarded);
                        crate::kernel::revoke_equip_skills(
                            ctx.core,
                            ctx.catalog,
                            self.owner,
                            discarded,
                        );
                        ctx.to_discard(discarded);
                    }
                    ctx.log(LogEvent::CardDiscarded {
                        player: self.owner,
                        cards: vec![discarded],
                    });
                    let frame = ctx.res.frame_mut(ctx.current_frame_id()).expect("frame");
                    if let EventData::Damage(dm) = &mut frame.data {
                        if let Some(amount) = dm.amount.checked_add(1) { dm.amount = amount; } else { ctx.fail_damage_amount(); return Flow::Done; }
                        dm.nature = crate::event::DamageNature::Fire;
                    }
                    Flow::Done
                }
                _ => Flow::Done,
            },
            // 缚斧：DamageBefore 帧直写 num+1（乱剑先例）
            HostBehavior::SwdJufu => {
                let frame = ctx.res.frame_mut(ctx.current_frame_id()).expect("frame");
                if let EventData::Damage(dm) = &mut frame.data {
                    if let Some(amount) = dm.amount.checked_add(1) { dm.amount = amount; } else { ctx.fail_damage_amount(); return Flow::Done; }
                }
                Flow::Done
            }
            // 破星：目标体力 > 持有者 → 伤害 +1（帧直写）
            HostBehavior::SwdPoxing => {
                if let Some(t) = self.target {
                    if ctx.core.player(t).hp > ctx.core.player(self.owner).hp {
                        let frame = ctx.res.frame_mut(ctx.current_frame_id()).expect("frame");
                        if let EventData::Damage(dm) = &mut frame.data {
                            if let Some(amount) = dm.amount.checked_add(1) { dm.amount = amount; } else { ctx.fail_damage_amount(); return Flow::Done; }
                        }
                    }
                }
                Flow::Done
            }
            // 神焰：前置面（+1 并记录蔓延目标）与结算面（逐个蔓延火伤）
            HostBehavior::SwdShenyan => {
                let (target, amount) = match ctx.current_damage() {
                    Ok(d) => (d.target, d.amount),
                    Err(_) => return Flow::Done,
                };
                if !ctx.core.player(self.owner).shenyan_used {
                    // 前置面：火伤 +1、置限定位、记录原目标距离 ≤1 的其他
                    // 角色（座位序，不含来源与原目标）
                    ctx.current_damage_mut().amount = amount + 1;
                    let mut pending: Vec<PlayerId> = ctx
                        .core
                        .seat_order_from(target)
                        .into_iter()
                        .filter(|p| {
                            *p != self.owner
                                && *p != target
                                && ctx.core.player(*p).alive
                                && ctx.core.computed_distance(ctx.catalog, target, *p) <= 1
                        })
                        .collect();
                    pending.sort_unstable();
                    let p = ctx.core.player_mut(self.owner);
                    p.shenyan_used = true;
                    p.shenyan_pending = pending;
                    ctx.log(LogEvent::SkillTriggered {
                        skill: "神焰",
                        owner: self.owner,
                    });
                } else {
                    // 结算面：逐个蔓延（上游 shenyan2 global damageAfter 循环）
                    let pending: Vec<PlayerId> =
                        std::mem::take(&mut ctx.core.player_mut(self.owner).shenyan_pending);
                    for t in pending {
                        if !ctx.core.player(t).alive {
                            continue;
                        }
                        ctx.emit(EventData::Damage(crate::machine::damage_data_with(
                            None,
                            t,
                            1,
                            crate::event::DamageNature::Fire,
                            None,
                        )));
                    }
                }
                Flow::Done
            }
            _ => unreachable!("behavior routed to the wrong step module"),
        }
    }
}
