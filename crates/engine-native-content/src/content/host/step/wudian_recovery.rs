use super::super::*;

impl HostMachine {
    pub(super) fn step_wudian_recovery(&mut self, ctx: &mut Ctx<'_>) -> Flow {
        match self.behavior {
            HostBehavior::WudianGuangshuHeart => {
                // ♥光书：受伤后回复 1 点体力（Damage 触发臂 filter 已
                // 收窄；一次性移除实例）。
                ctx.recover(self.owner, 1, None);
                if let Some(def) = ctx.catalog.skill_by_name(WUDIAN_GUANGSHU_HEART_KEY) {
                    ctx.core.skills.remove(&(self.owner, def));
                }
                Flow::Done
            }
            HostBehavior::WudianZiyu => match self.stage {
                // 自愈（U07 ow，规则 v120）：摸一张或回复 1 点体力（可放
                // 弃；满血时无回复选项——上游 chooseDrawRecover 同口径）。
                HostStage::Act => {
                    let p = ctx.core.player(self.owner);
                    let mut options = vec![ChoiceOption::Tag(0)];
                    if (p.hp as u32) < p.max_hp {
                        options.push(ChoiceOption::Tag(1));
                    }
                    self.stage = HostStage::SelectCard;
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::PickTag,
                            prompt: WUDIAN_ZIYU_CHOICE,
                            options,
                            min: 1,
                            max: 1,
                            allow_cancel: true,
                        },
                    }
                }
                HostStage::SelectCard => match ctx.answer().first() {
                    Some(ChoiceOption::Tag(1)) => {
                        ctx.recover(self.owner, 1, None);
                        Flow::Done
                    }
                    Some(ChoiceOption::Tag(0)) => {
                        ctx.start_draw(self.owner, 1);
                        Flow::Done
                    }
                    _ => Flow::Done,
                },
                _ => Flow::Done,
            },
            HostBehavior::WudianShouhu => match self.stage {
                // 守护（U07 ow，规则 v121）：Act 弃一张杀 → SelectCard 选
                // 其他受伤角色 → 回复 1 点体力（上游 target.recover 同口
                // 径）。
                HostStage::Act => {
                    let options: Vec<ChoiceOption> = ctx
                        .core
                        .player(self.owner)
                        .hand
                        .iter()
                        .copied()
                        .filter(|c| ctx.catalog.card_kind(*c).is_sha_like())
                        .map(ChoiceOption::Card)
                        .collect();
                    self.stage = HostStage::SelectCard;
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectCards,
                            prompt: WUDIAN_SHOUHU_CARD,
                            options,
                            min: 1,
                            max: 1,
                            allow_cancel: false,
                        },
                    }
                }
                HostStage::SelectCard => {
                    let Some(c) = ctx.answer().first().and_then(|o| match o {
                        ChoiceOption::Card(c) => Some(*c),
                        _ => None,
                    }) else {
                        return Flow::Done;
                    };
                    if !ctx.core.player(self.owner).hand.contains(&c)
                        || !ctx.catalog.card_kind(c).is_sha_like()
                    {
                        return Flow::Done;
                    }
                    ctx.discard_from_hand(self.owner, c);
                    ctx.log(LogEvent::CardDiscarded {
                        player: self.owner,
                        cards: vec![c],
                    });
                    let options: Vec<ChoiceOption> = ctx
                        .core
                        .alive_players()
                        .filter(|p| {
                            *p != self.owner
                                && (ctx.core.player(*p).hp as u32) < ctx.core.player(*p).max_hp
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
                            prompt: WUDIAN_SHOUHU_TARGET,
                            options,
                            min: 1,
                            max: 1,
                            allow_cancel: false,
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
                        ctx.recover(t, 1, None);
                    }
                    Flow::Done
                }
                _ => Flow::Done,
            },
            HostBehavior::WudianZhenchaDraw => match self.stage {
                // 侦察摸回（U05 ow，规则 v125）：摸一张或回复 1 点体力
                //（chooseDrawRecover 可放弃；满血仅摸牌）。
                HostStage::Act => {
                    let p = ctx.core.player(self.owner);
                    let mut options = vec![ChoiceOption::Tag(0)];
                    if (p.hp as u32) < p.max_hp {
                        options.push(ChoiceOption::Tag(1));
                    }
                    self.stage = HostStage::SelectCard;
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::PickTag,
                            prompt: WUDIAN_ZHENCHA_CHOICE,
                            options,
                            min: 1,
                            max: 1,
                            allow_cancel: true,
                        },
                    }
                }
                HostStage::SelectCard => match ctx.answer().first() {
                    Some(ChoiceOption::Tag(1)) => {
                        ctx.recover(self.owner, 1, None);
                        Flow::Done
                    }
                    Some(ChoiceOption::Tag(0)) => {
                        ctx.start_draw(self.owner, 1);
                        Flow::Done
                    }
                    _ => Flow::Done,
                },
                _ => Flow::Done,
            },
            HostBehavior::WudianYuedong => match self.stage {
                // 乐动（U05 ow 卢西奥，规则 v126）：出牌阶段结束时选至多
                // 1+扩音增量名目标各摸 1+扩音量张；呼唤后改为各回复等量
                //（回复模式目标须未满体力；上游 direct + chooseTarget
                // [1,num] 可整体放弃——空选项直接跳过）。
                HostStage::Act => {
                    let num = 1 + ctx.core.turn.yuedong_num as usize;
                    let recover = ctx.core.turn.yuedong_recover;
                    let options: Vec<ChoiceOption> = ctx
                        .core
                        .alive_players()
                        .filter(|p| {
                            !recover || (ctx.core.player(*p).hp as u32) < ctx.core.player(*p).max_hp
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
                            prompt: WUDIAN_YUEDONG_TARGETS,
                            options,
                            min: 1,
                            max: num,
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
                    if targets.is_empty() {
                        return Flow::Done; // 整体放弃
                    }
                    let eff = 1 + ctx.core.turn.yuedong_eff as usize;
                    if ctx.core.turn.yuedong_recover {
                        // 上游按座位序逐个回复（sort seat；原生座位序同构）
                        let mut sorted = targets.clone();
                        sorted.sort_by_key(|p| ctx.core.seat_steps_from(ctx.core.turn.current, *p));
                        for p in sorted {
                            ctx.recover(p, eff as u32, None);
                        }
                    } else {
                        for p in targets {
                            ctx.start_draw(p, eff as u32);
                        }
                    }
                    Flow::Done
                }
                _ => Flow::Done,
            },
            HostBehavior::WudianJidong => match self.stage {
                // 急冻（U05 ow 美，规则 v127）：翻面+回复 2+冰封。登记
                // 差异：翻面内联（风雪刃 v95 先例，不经 turnOverBefore
                // 窗口）；冰封解除=翻回时点（commit_turn_over_before 翻
                // 至正面清位，上游 turnOverAfter 到期同口径）。
                HostStage::Act => {
                    let flipped = {
                        let p = ctx.core.player_mut(self.owner);
                        p.face_down = !p.face_down;
                        p.face_down
                    };
                    ctx.emit(EventData::TurnOvered {
                        player: self.owner,
                        face_down: flipped,
                    });
                    ctx.recover(self.owner, 2, None);
                    if flipped {
                        ctx.core.player_mut(self.owner).jidong_active = true;
                    }
                    Flow::Done
                }
                _ => Flow::Done,
            },
            HostBehavior::WudianXiyang => match self.stage {
                // 吸阳（U05 ow 路霸，规则 v128）：确认后翻面+回复 2。
                // 登记差异：翻面内联（风雪刃 v95 先例，不经窗口——与
                // 急冻同口径）。
                HostStage::Act => {
                    let flipped = {
                        let p = ctx.core.player_mut(self.owner);
                        p.face_down = !p.face_down;
                        p.face_down
                    };
                    ctx.emit(EventData::TurnOvered {
                        player: self.owner,
                        face_down: flipped,
                    });
                    ctx.recover(self.owner, 2, None);
                    Flow::Done
                }
                _ => Flow::Done,
            },
            HostBehavior::WudianZhiyuan => match self.stage {
                // 支援（U05 ow 安娜，规则 v131）：防止此伤害，受伤者回
                // 复等量体力（上游 trigger.cancel()+recover(num) 同口
                // 径；确认门由触发路径 OptionalGate 承担）。
                HostStage::Act => {
                    let (num, hurt) = {
                        let d = ctx.current_damage_mut();
                        d.prevented = true;
                        (d.amount, d.target)
                    };
                    if num > 0 && ctx.core.player(hurt).alive {
                        ctx.recover(hurt, num.max(0) as u32, None);
                    }
                    Flow::Done
                }
                _ => Flow::Done,
            },
            _ => unreachable!("behavior routed to the wrong step module"),
        }
    }
}
