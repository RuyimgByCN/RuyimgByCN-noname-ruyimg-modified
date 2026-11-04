use super::super::*;

impl HostMachine {
    pub(super) fn step_yunchou(&mut self, ctx: &mut Ctx<'_>) -> Flow {
        match self.behavior {
            // 厉焰：目录锚（转化逻辑在 PlayCard 门，无独立效果步）
            HostBehavior::YunchouLuweiyan => Flow::Done,
            // 雄姿：补摸 1+⌊装备数/2⌋
            HostBehavior::YunchouXiongzi => {
                let n = 1 + ctx.core.player(self.owner).equips.len() / 2;
                ctx.start_draw(self.owner, n as u32);
                Flow::Done
            }
            // 雅量：伤害双方各摸 1（座位序逐个；R05-C2 逆序压帧＝
            // 同步升序执行/取牌次序）
            HostBehavior::YunchouYaliang => {
                let mut seats = self.targets.clone();
                seats.sort_unstable();
                seats.dedup();
                seats.retain(|s| ctx.core.player(*s).alive);
                for s in seats.iter().rev() {
                    ctx.start_draw(*s, 1);
                }
                Flow::Done
            }
            // 蝉缘·变体：来源体力上限 -1（体力钳上限）
            HostBehavior::YunchouDiychanyuan => {
                if let Some(src) = self.target {
                    ctx.lose_max_hp(src, 1);
                }
                Flow::Done
            }
            // 悲歌（refresh 变体）：判定→弃 1 张（可取消）→按判定花色
            // 分支+收回奖励。登记差异：草花来源弃 2 取首两张确定性；
            // addExpose（AI 面）不做。
            HostBehavior::YunchouBeige => match self.stage {
                HostStage::Act => {
                    // 受伤者=targets[1]，来源=targets[0]
                    let judged = ctx.draw_judge_card();
                    ctx.to_discard(judged);
                    self.cards = vec![judged];
                    self.stage = HostStage::SelectCard;
                    let options: Vec<ChoiceOption> = ctx
                        .core
                        .player(self.owner)
                        .hand
                        .iter()
                        .chain(ctx.core.player(self.owner).equips.iter())
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
                            prompt: "beige_discard",
                            options,
                            min: 0,
                            max: 1,
                            allow_cancel: true,
                        },
                    }
                }
                HostStage::SelectCard => {
                    let [src, target] = [self.targets[0], self.targets[1]];
                    let judged = self.cards[0];
                    let judged_suit = ctx.catalog.cards[judged as usize].suit;
                    let picked: Option<CardId> = ctx.answer().iter().find_map(|o| match o {
                        ChoiceOption::Card(c) => Some(*c),
                        _ => None,
                    });
                    let Some(discarded) = picked else {
                        // 取消：判定牌已在弃堆，无效果
                        return Flow::Done;
                    };
                    // 弃置所选（手/装备）
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
                    // 按判定花色分支
                    let mut diamond_drew = false;
                    match judged_suit {
                        crate::catalog::Suit::Heart => {
                            let p = ctx.core.player(target);
                            if p.alive && p.hp < p.max_hp as i32 {
                                ctx.recover(target, 1, None);
                            }
                        }
                        crate::catalog::Suit::Diamond => {
                            if ctx.core.player(target).alive {
                                // R05-C2：方块摸 2 发 R 帧后续行（同步摸
                                // 先于收回奖励读弃堆——洗牌边界下序敏感）。
                                ctx.start_draw(target, 2);
                                diamond_drew = true;
                            }
                        }
                        crate::catalog::Suit::Spade => {
                            if ctx.core.player(src).alive {
                                let flipped = {
                                    let p = ctx.core.player_mut(src);
                                    p.face_down = !p.face_down;
                                    p.face_down
                                };
                                ctx.emit(EventData::TurnOvered {
                                    player: src,
                                    face_down: flipped,
                                });
                            }
                        }
                        crate::catalog::Suit::Club => {
                            // 来源弃 he 前 2 张（确定性首两张，登记差异）
                            if ctx.core.player(src).alive {
                                let drop: Vec<CardId> = ctx
                                    .core
                                    .player(src)
                                    .hand
                                    .iter()
                                    .chain(ctx.core.player(src).equips.iter())
                                    .copied()
                                    .take(2)
                                    .collect();
                                for c in &drop {
                                    if ctx.core.player(src).hand.contains(c) {
                                        ctx.discard_from_hand(src, *c);
                                    } else {
                                        ctx.core.player_mut(src).equips.retain(|x| x != c);
                                        crate::kernel::revoke_equip_skills(
                                            ctx.core,
                                            ctx.catalog,
                                            src,
                                            *c,
                                        );
                                        ctx.to_discard(*c);
                                    }
                                    ctx.log(LogEvent::CardDiscarded {
                                        player: src,
                                        cards: vec![*c],
                                    });
                                }
                            }
                        }
                    }
                    if diamond_drew {
                        // 续行位：R 子树弹栈后做收回奖励（同步序＝摸先于奖励）。
                        self.cards = vec![judged, discarded];
                        self.stage = HostStage::DrawSettle;
                        return Flow::Yield;
                    }
                    beige_gains(ctx, self.owner, judged, discarded);
                    Flow::Done
                }
                // R05-C2 续行位：悲歌方块臂摸 2 交付后的收回奖励尾段。
                HostStage::DrawSettle => {
                    let &[judged, discarded] = &self.cards[..] else {
                        return Flow::Done;
                    };
                    beige_gains(ctx, self.owner, judged, discarded);
                    Flow::Done
                }
                _ => Flow::Done,
            },
            // 隙焰：弃首张红手牌（登记差异）→ 全体其他存活座位序各 1 火
            HostBehavior::YunchouJieyan => {
                let red = ctx.core.player(self.owner).hand.iter().copied().find(|c| {
                    matches!(
                        ctx.catalog.cards[*c as usize].suit,
                        crate::catalog::Suit::Heart | crate::catalog::Suit::Diamond
                    )
                });
                let Some(c) = red else { return Flow::Done };
                ctx.discard_from_hand(self.owner, c);
                ctx.log(LogEvent::CardDiscarded {
                    player: self.owner,
                    cards: vec![c],
                });
                let seats: Vec<u8> = (0..ctx.core.players.len() as u8)
                    .filter(|&i| i != self.owner && ctx.core.player(i).alive)
                    .collect();
                for t in seats {
                    ctx.emit(crate::event::EventData::Damage(
                        crate::machine::damage_data_with(
                            Some(self.owner),
                            t,
                            1,
                            crate::event::DamageNature::Fire,
                            None,
                        ),
                    ));
                }
                Flow::Done
            }
            // 野升：自选任意张黑手牌 → 弃所选摸等量（舒筋散机器同构）
            HostBehavior::YunchouYiesheng => match self.stage {
                HostStage::SelectCard => {
                    let options: Vec<ChoiceOption> = ctx
                        .core
                        .player(self.owner)
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
                        return Flow::Done;
                    }
                    let n = options.len();
                    self.stage = HostStage::Act;
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectCards,
                            prompt: "yiesheng_discard",
                            options,
                            min: 1,
                            max: n,
                            allow_cancel: true,
                        },
                    }
                }
                HostStage::Act => {
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
                            ctx.core.player_mut(self.owner).hand.retain(|x| x != c);
                        }
                        ctx.log(LogEvent::CardDiscarded {
                            player: self.owner,
                            cards: picked.clone(),
                        });
                        // R05-C2：弃后摸等量——步末无后续读，R 帧直接续
                        //（洗牌含刚弃牌的边界与同步一致：R 泵在步 Done 后
                        //立即发生，中间无其他结算）。
                        ctx.start_draw(self.owner, picked.len() as u32);
                    }
                    Flow::Done
                }
                _ => Flow::Done,
            },
            // 厉焰·链杀：对座位序首目标视为用杀（虚拟杀直发；登记差异：
            // 上游玩家选目标+可取消，原生自动首目标）
            HostBehavior::YunchouLuweiyan2 => {
                let mut seats = self.targets.clone();
                seats.sort_unstable();
                seats.dedup();
                if let Some(t) = seats
                    .into_iter()
                    .find(|&t| t != self.owner && ctx.core.player(t).alive)
                {
                    ctx.use_virtual_card(self.owner, crate::catalog::CardKind::Sha, vec![t]);
                }
                Flow::Done
            }
            // 蛊惑·变体：摸 2 然后弃自己区域 2 张（判定区→装备→手牌首张序）
            HostBehavior::YunchouDiyguhuo => match self.stage {
                HostStage::Act => {
                    // R05-C2：摸 2 走 R 帧；「判定区→装备→手牌首张」的
                    // 退弃依赖摸后手牌（空手时同步会弃掉刚摸的牌）→ 续行位。
                    ctx.start_draw(self.owner, 2);
                    self.stage = HostStage::DrawSettle;
                    Flow::Yield
                }
                HostStage::DrawSettle => {
                    for _ in 0..2 {
                        let next = if let Some(c) = ctx.core.player(self.owner).judge.first().copied() {
                            ctx.core.player_mut(self.owner).judge.retain(|x| *x != c);
                            Some(c)
                        } else if let Some(c) = ctx.core.player(self.owner).equips.first().copied() {
                            ctx.core.player_mut(self.owner).equips.retain(|x| *x != c);
                            crate::kernel::revoke_equip_skills(ctx.core, ctx.catalog, self.owner, c);
                            Some(c)
                        } else {
                            ctx.core.player(self.owner).hand.first().copied()
                        };
                        if let Some(c) = next {
                            ctx.core.player_mut(self.owner).hand.retain(|x| *x != c);
                            ctx.to_discard(c);
                            ctx.log(LogEvent::CardDiscarded {
                                player: self.owner,
                                cards: vec![c],
                            });
                        }
                    }
                    Flow::Done
                }
                _ => Flow::Done,
            },
            // 狡黠：摸 1
            HostBehavior::YunchouJiaoxia => {
                ctx.start_draw(self.owner, 1);
                Flow::Done
            }
            // 红莲：来源弃其手牌与装备区全部红牌（装备走 revoke 同款）
            HostBehavior::YunchouHonglian => {
                let src = match self.target {
                    Some(t) => t,
                    None => return Flow::Done,
                };
                let reds: Vec<_> = {
                    let p = ctx.core.player(src);
                    p.hand
                        .iter()
                        .chain(p.equips.iter())
                        .copied()
                        .filter(|c| {
                            matches!(
                                ctx.catalog.cards[*c as usize].suit,
                                crate::catalog::Suit::Heart | crate::catalog::Suit::Diamond
                            )
                        })
                        .collect()
                };
                for c in reds {
                    if ctx.core.player(src).hand.contains(&c) {
                        ctx.discard_from_hand(src, c);
                    } else {
                        ctx.core.player_mut(src).equips.retain(|x| *x != c);
                        crate::kernel::revoke_equip_skills(ctx.core, ctx.catalog, src, c);
                        ctx.to_discard(c);
                    }
                    ctx.log(LogEvent::CardDiscarded {
                        player: src,
                        cards: vec![c],
                    });
                }
                Flow::Done
            }
            // 纵火：弃首张手牌（登记差异）→ 直写帧 nature=Fire
            HostBehavior::YunchouZonghuo => {
                let hand = ctx.core.player(self.owner).hand.clone();
                if let Some(c) = hand.first().copied() {
                    ctx.discard_from_hand(self.owner, c);
                    ctx.log(LogEvent::CardDiscarded {
                        player: self.owner,
                        cards: vec![c],
                    });
                    let frame = ctx
                        .res
                        .frame_mut(ctx.current_event_frame_id())
                        .expect("frame");
                    if let EventData::Damage(dm) = &mut frame.data {
                        dm.nature = crate::event::DamageNature::Fire;
                    }
                }
                Flow::Done
            }
            // 烧营：座位序首合法目标（受伤者距离 ≤1 且非受伤者）→ 弃堆顶
            // 牌亮出，红色则其受 1 点火伤（登记差异：上游玩家选目标）
            HostBehavior::YunchouShaoying => {
                let hurt = match self.target {
                    Some(t) => t,
                    None => return Flow::Done,
                };
                let target = (0..ctx.core.players.len()).map(|i| i as u8).find(|&i| {
                    i != hurt
                        && i != self.owner
                        && ctx.core.player(i).alive
                        && ctx.core.seat_distance(hurt, i) <= 1
                });
                let Some(t) = target else { return Flow::Done };
                let Some(card) = ctx.core.deck.pop() else {
                    return Flow::Done;
                };
                let red = matches!(
                    ctx.catalog.cards[card as usize].suit,
                    crate::catalog::Suit::Heart | crate::catalog::Suit::Diamond
                );
                ctx.to_discard(card);
                ctx.log(LogEvent::CardDiscarded {
                    player: self.owner,
                    cards: vec![card],
                });
                if red {
                    ctx.emit(crate::event::EventData::Damage(
                        crate::machine::damage_data_with(
                            Some(self.owner),
                            t,
                            1,
                            crate::event::DamageNature::Fire,
                            None,
                        ),
                    ));
                }
                Flow::Done
            }
            // 戏才：获得伤害牌（table 优先、弃堆兜底；登记近似：不校验
            // 上游 position 'o' 处理区位）
            HostBehavior::YunchouXicai => {
                if let Some(c) = self.card {
                    let taken = if let Some(pos) = ctx.core.table.iter().position(|x| *x == c) {
                        ctx.core.table.remove(pos);
                        true
                    } else if let Some(pos) = ctx.core.discard.iter().position(|x| *x == c) {
                        ctx.core.discard.remove(pos);
                        true
                    } else {
                        false
                    };
                    if taken {
                        ctx.core.player_mut(self.owner).hand.push(c);
                        ctx.log(LogEvent::CardsGivenPublic {
                            from: self.owner,
                            to: self.owner,
                            cards: vec![c],
                        });
                    }
                }
                Flow::Done
            }
            // 狂骨·变体：目标攻击距离 >1 摸 1，否则已受伤回复 1
            HostBehavior::YunchouDiykuanggu => {
                if let Some(t) = self.target {
                    if ctx.core.seat_distance(t, self.owner) > 1 {
                        ctx.start_draw(self.owner, 1);
                    } else {
                        let p = ctx.core.player(self.owner);
                        if p.hp < p.max_hp as i32 {
                            ctx.recover(self.owner, 1, None);
                        }
                    }
                }
                Flow::Done
            }
            _ => unreachable!("behavior routed to the wrong step module"),
        }
    }
}

/// 悲歌收回奖励（R05-C2 抽出：花色相同→判定牌归持有者；点数相同→
/// 所弃牌归持有者。SelectCard 即时段与 DrawSettle 续行段共用）。
fn beige_gains(ctx: &mut Ctx<'_>, owner: PlayerId, judged: CardId, discarded: CardId) {
    let judged_suit = ctx.catalog.cards[judged as usize].suit;
    let judged_num = ctx.catalog.cards[judged as usize].rank;
    let disc_suit = ctx.catalog.cards[discarded as usize].suit;
    let disc_num = ctx.catalog.cards[discarded as usize].rank;
    let mut gains: Vec<CardId> = Vec::new();
    if disc_suit == judged_suit {
        if let Some(pos) = ctx.core.discard.iter().position(|x| *x == judged) {
            ctx.core.discard.remove(pos);
            gains.push(judged);
        }
    }
    if disc_num == judged_num && discarded != judged {
        if let Some(pos) = ctx.core.discard.iter().position(|x| *x == discarded) {
            ctx.core.discard.remove(pos);
            gains.push(discarded);
        }
    }
    if !gains.is_empty() {
        ctx.core.player_mut(owner).hand.extend(gains.iter().copied());
        ctx.log(LogEvent::CardsGivenPublic {
            from: owner,
            to: owner,
            cards: gains,
        });
    }
}
