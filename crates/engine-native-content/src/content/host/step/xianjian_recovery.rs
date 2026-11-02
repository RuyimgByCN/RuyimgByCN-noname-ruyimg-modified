use super::super::*;

impl HostMachine {
    pub(super) fn step_xianjian_recovery(&mut self, ctx: &mut Ctx<'_>) -> Flow {
        match self.behavior {
            HostBehavior::XianjianTianshe2 => match self.stage {
                // 蛇蜕·息（U05 xianjian 赵灵儿，规则 v144）：回复 1 点体
                // 力（满血饱和；filter 已收窄已受伤）。
                HostStage::Act => {
                    ctx.recover(self.owner, 1, None);
                    Flow::Done
                }
                _ => Flow::Done,
            },
            HostBehavior::XianjianFanling => match self.stage {
                // 返灵（U05 xianjian 瑕，规则 v145）：回复 1 点体力+置
                // 本回合已发动（换回合复位）。
                HostStage::Act => {
                    ctx.core.turn.fanling_used = true;
                    ctx.recover(self.owner, 1, None);
                    Flow::Done
                }
                _ => Flow::Done,
            },
            HostBehavior::XianjianGuiyuan => match self.stage {
                // 归元（U05 xianjian 林月如，规则 v147）：弃 1 张杀（必
                // 选）→回复 1 点体力+摸一张牌（已受伤 filter 收窄）。
                HostStage::SelectCard => {
                    let p = ctx.core.player(self.owner);
                    let options: Vec<ChoiceOption> = p
                        .hand
                        .iter()
                        .copied()
                        .chain(p.equips.iter().copied())
                        .filter(|c| ctx.catalog.card_kind(*c).is_sha_like())
                        .map(ChoiceOption::Card)
                        .collect();
                    if options.is_empty()
                        || (ctx.core.player(self.owner).hp as u32)
                            >= ctx.core.player(self.owner).max_hp
                    {
                        return Flow::Done;
                    }
                    self.stage = HostStage::Act;
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectCards,
                            prompt: WUDIAN_XJ_GUIYUAN_CARDS,
                            options,
                            min: 1,
                            max: 1,
                            allow_cancel: false,
                        },
                    }
                }
                HostStage::Act => {
                    let Some(c) = ctx.answer().first().and_then(|o| match o {
                        ChoiceOption::Card(c) => Some(*c),
                        _ => None,
                    }) else {
                        return Flow::Done;
                    };
                    ctx.take_owned_card(self.owner, c);
                    ctx.to_discard(c);
                    ctx.log(LogEvent::CardDiscarded {
                        player: self.owner,
                        cards: vec![c],
                    });
                    ctx.recover(self.owner, 1, None);
                    ctx.draw_cards(self.owner, 1);
                    Flow::Done
                }
                _ => Flow::Done,
            },
            HostBehavior::XianjianNingbin => match self.stage {
                // 凝冰（U05 xianjian 玄霄，规则 v152）：回复 1 点体力（满
                // 血饱和）。
                HostStage::Act => {
                    ctx.recover(self.owner, 1, None);
                    Flow::Done
                }
                _ => Flow::Done,
            },
            HostBehavior::XianjianLingyu => match self.stage {
                // 灵愈（U05 xianjian 龙葵，规则 v153）：令一名其他已受伤
                // 角色回复 1 点体力（可放弃=空目标）。
                HostStage::Act => {
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
                            prompt: WUDIAN_XJ_LINGYU_TARGET,
                            options,
                            min: 1,
                            max: 1,
                            allow_cancel: true,
                        },
                    }
                }
                HostStage::TargetChoice => {
                    if let Some(ChoiceOption::Player(t)) = ctx.answer().first() {
                        let t = *t;
                        if ctx.core.player(t).alive
                            && (ctx.core.player(t).hp as u32) < ctx.core.player(t).max_hp
                        {
                            ctx.recover(t, 1, None);
                        }
                    }
                    Flow::Done
                }
                _ => Flow::Done,
            },
            HostBehavior::XianjianTxianqu => match self.stage {
                // 甜香去（U05 xianjian 唐雨柔，规则 v156）：防止伤害（安
                // 娜支援同款 prevented 直写）+摸 2+回复 1+置本回合已发动。
                HostStage::Act => {
                    ctx.current_damage_mut().prevented = true;
                    ctx.core.turn.txianqu_used = true;
                    ctx.draw_cards(self.owner, 2);
                    ctx.recover(self.owner, 1, None);
                    Flow::Done
                }
                _ => Flow::Done,
            },
            HostBehavior::XianjianSajin => match self.stage {
                // 洒金（U05 xianjian 景天，规则 v158）：弃 1 张手牌（必
                // 选）→ 指定任意名已受伤角色（上游 selectTarget
                // [1,Infinity] 同口径）→ 逐目标判定（Yield 后重查），
                // 判定颜色与所弃牌相同者回复 1 点体力。每臂只收一个应答：
                // SelectCard 发问、TargetChoice 收卡+发问、Act 收目标
                //（self.card 兼首入标记）+弃牌+纯循环。
                HostStage::SelectCard => {
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
                    self.stage = HostStage::TargetChoice;
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectCards,
                            prompt: WUDIAN_XJ_SAJIN_CARDS,
                            options,
                            min: 1,
                            max: 1,
                            allow_cancel: false,
                        },
                    }
                }
                HostStage::TargetChoice => {
                    if let Some(ChoiceOption::Card(c)) = ctx.answer().first() {
                        self.card = Some(*c);
                    }
                    let Some(c) = self.card else {
                        return Flow::Done;
                    };
                    let options: Vec<ChoiceOption> = ctx
                        .core
                        .alive_players()
                        .filter(|t| (ctx.core.player(*t).hp as u32) < ctx.core.player(*t).max_hp)
                        .map(ChoiceOption::Player)
                        .collect();
                    if options.is_empty() {
                        // 无可指定目标：牌照弃（已进入技能，上游同弃）
                        ctx.take_owned_card(self.owner, c);
                        ctx.to_discard(c);
                        ctx.log(LogEvent::CardDiscarded {
                            player: self.owner,
                            cards: vec![c],
                        });
                        return Flow::Done;
                    }
                    let max = options.len();
                    self.stage = HostStage::Act;
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectPlayers,
                            prompt: WUDIAN_XJ_SAJIN_TARGETS,
                            options,
                            min: 1,
                            max,
                            allow_cancel: false,
                        },
                    }
                }
                HostStage::Act => {
                    // 首入（self.card 尚存）：收目标应答并弃牌（弃牌色
                    // 弃后仍可读目录）；后续 Yield 重入只走逐目标循环
                    if let Some(c) = self.card.take() {
                        let picked: Vec<PlayerId> = ctx
                            .answer()
                            .iter()
                            .filter_map(|o| match o {
                                ChoiceOption::Player(p) => Some(*p),
                                _ => None,
                            })
                            .collect();
                        ctx.take_owned_card(self.owner, c);
                        ctx.to_discard(c);
                        ctx.log(LogEvent::CardDiscarded {
                            player: self.owner,
                            cards: vec![c],
                        });
                        self.targets = picked;
                        self.cards = vec![c];
                    }
                    // 逐目标：判定→同色回复（Yield 后重查存活）
                    let Some(t) = self.targets.first().copied() else {
                        return Flow::Done;
                    };
                    self.targets.remove(0);
                    if !ctx.core.player(t).alive {
                        return Flow::Yield;
                    }
                    let judged = ctx.draw_judge_card();
                    let suit = ctx.catalog.cards[judged as usize].suit;
                    ctx.log(LogEvent::Judged {
                        player: t,
                        card: judged,
                    });
                    ctx.to_discard(judged);
                    if let Some(c) = self.cards.first().copied() {
                        let is_red = |su: crate::catalog::Suit| {
                            matches!(
                                su,
                                crate::catalog::Suit::Heart | crate::catalog::Suit::Diamond
                            )
                        };
                        let disc_suit = ctx.catalog.cards[c as usize].suit;
                        if is_red(suit) == is_red(disc_suit) && ctx.core.player(t).alive {
                            ctx.recover(t, 1, None);
                        }
                    }
                    Flow::Yield
                }
                _ => Flow::Done,
            },
            HostBehavior::XianjianShuiyun5 => match self.stage {
                // 水云·救（U05 xianjian 紫萱，规则 v162）：濒死确认门后
                // 必选离场区一张牌弃置，濒死者回复 1（shuiyun_count++——
                // 小蛮 shenwu 链依赖）。target=机器分流带入的濒死者。
                HostStage::Act => {
                    let options: Vec<ChoiceOption> = ctx
                        .core
                        .player(self.owner)
                        .off_court
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
                            prompt: WUDIAN_XJ_SHUIYUN5_CARDS,
                            options,
                            min: 1,
                            max: 1,
                            allow_cancel: false,
                        },
                    }
                }
                HostStage::SelectCard => {
                    let Some(ChoiceOption::Card(c)) = ctx.answer().first() else {
                        return Flow::Done;
                    };
                    let c = *c;
                    ctx.off_court_discard(self.owner, c);
                    let t = self.target.unwrap_or(self.owner);
                    if ctx.core.player(t).alive || ctx.core.player(t).hp <= 0 {
                        ctx.recover(t, 1, None);
                    }
                    ctx.core.player_mut(self.owner).shuiyun_count += 1;
                    Flow::Done
                }
                _ => Flow::Done,
            },
            HostBehavior::XianjianShenwu => {
                // 神雾（U05 xianjian 小蛮，规则 v163）：觉醒——体力上限
                // +1、回复 1、runtime 授回梦实例（上游 gainMaxHp/recover/
                // addSkill("huimeng") 同口径）。
                ctx.core.player_mut(self.owner).shenwu_awakened = true;
                ctx.core.player_mut(self.owner).max_hp += 1;
                ctx.recover(self.owner, 1, None);
                if let Some(def) = ctx.catalog.skill_by_name("huimeng") {
                    ctx.insert_skill(
                        (self.owner, def),
                        crate::state::SkillInstance {
                            owner: self.owner,
                            enabled: true,
                            once_used: false,
                            expires_end_of_turn: false,
                            rounds: 0,
                        },
                    );
                }
                Flow::Done
            }
            HostBehavior::XianjianChengxin => {
                // 诚信（U05 xianjian 明绣，规则 v164）：确认后冷却置位+
                // 濒死者回复至 1（recover 量=1-当前体力）+免伤至其 End
                // 阶段结束（kernel 直钩）。target=分流带入的濒死者。
                ctx.core.player_mut(self.owner).chengxin_ready_round = ctx.core.turn.round + 4;
                if let Some(t) = self.target {
                    let hp = ctx.core.player(t).hp;
                    if hp <= 0 {
                        let amount = (1 - hp) as u32;
                        ctx.recover(t, amount, None);
                        ctx.core.player_mut(t).chengxin_active = true;
                    }
                }
                Flow::Done
            }
            HostBehavior::XianjianShenmu => {
                // 神暮（U05 xianjian 雪见，规则 v165）：确认后弃全部红手
                // 牌，濒死者回复 1 并摸等量张（上游 showHandcards 纯表现
                // 不迁移）。target=分流带入的濒死者。
                let reds: Vec<CardId> = ctx
                    .core
                    .player(self.owner)
                    .hand
                    .iter()
                    .copied()
                    .filter(|c| {
                        matches!(
                            ctx.catalog.cards[*c as usize].suit,
                            crate::catalog::Suit::Heart | crate::catalog::Suit::Diamond
                        )
                    })
                    .collect();
                let n = reds.len();
                for c in reds {
                    ctx.discard_from_hand(self.owner, c);
                }
                if let Some(t) = self.target {
                    if ctx.core.player(t).hp <= 0 {
                        ctx.recover(t, 1, None);
                        ctx.draw_cards(t, n);
                    }
                }
                Flow::Done
            }
            HostBehavior::XianjianRunxin => match self.stage {
                // 润心（U05 xianjian 柳梦璃，规则 v166）：使用/打出红桃
                // 牌后令一名已受伤角色回复 1（chooseTarget 取消=放弃）。
                // 首臂发问。
                HostStage::Act => {
                    let options: Vec<ChoiceOption> = ctx
                        .core
                        .alive_players()
                        .filter(|p| (ctx.core.player(*p).hp as u32) < ctx.core.player(*p).max_hp)
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
                            prompt: WUDIAN_XJ_RUNXIN_TARGET,
                            options,
                            min: 1,
                            max: 1,
                            allow_cancel: true,
                        },
                    }
                }
                HostStage::TargetChoice => {
                    if let Some(ChoiceOption::Player(t)) = ctx.answer().first() {
                        let t = *t;
                        if (ctx.core.player(t).hp as u32) < ctx.core.player(t).max_hp {
                            ctx.recover(t, 1, None);
                        }
                    }
                    Flow::Done
                }
                _ => Flow::Done,
            },
            HostBehavior::XianjianHuahu => match self.stage {
                // 花好（U05 xianjian 苏媚，规则 v175）：多目标选择→置一次
                // 性+失 1 体力上限+clearSkills 自己→逐目标 Yield：各回复
                // 1+获 1 护甲+摸 1。首臂发问。
                HostStage::TargetChoice => {
                    let options: Vec<ChoiceOption> = ctx
                        .core
                        .alive_players()
                        .filter(|p| *p != self.owner)
                        .map(ChoiceOption::Player)
                        .collect();
                    if options.is_empty() {
                        return Flow::Done;
                    }
                    let max = options.len();
                    self.stage = HostStage::Act;
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectPlayers,
                            prompt: WUDIAN_XJ_HUAHU_TARGETS,
                            options,
                            min: 1,
                            max,
                            allow_cancel: true,
                        },
                    }
                }
                HostStage::Act => {
                    if self.target.is_none() {
                        self.target = Some(self.owner); // 装填标记
                        let picked: Vec<PlayerId> = ctx
                            .answer()
                            .iter()
                            .filter_map(|o| match o {
                                ChoiceOption::Player(p) => Some(*p),
                                _ => None,
                            })
                            .collect();
                        self.targets = picked;
                        ctx.core.player_mut(self.owner).huahu_used = true;
                        let pl = ctx.core.player_mut(self.owner);
                        pl.max_hp = pl.max_hp.saturating_sub(1);
                        let mh = pl.max_hp as i32;
                        if pl.hp > mh {
                            pl.hp = mh;
                        }
                        let removed: Vec<crate::ids::SkillDefId> = ctx
                            .core
                            .skills
                            .keys()
                            .filter(|(o, _)| *o == self.owner)
                            .map(|(_, d)| *d)
                            .collect();
                        for def in removed {
                            ctx.core.skills.remove(&(self.owner, def));
                            ctx.log(LogEvent::SkillToggled {
                                player: self.owner,
                                skill: ctx.catalog.skill_name(def),
                                enabled: false,
                            });
                        }
                    }
                    let Some(t) = self.targets.first().copied() else {
                        return Flow::Done;
                    };
                    self.targets.remove(0);
                    if ctx.core.player(t).alive {
                        ctx.recover(t, 1, None);
                        ctx.change_hujia(t, 1);
                        ctx.draw_cards(t, 1);
                    }
                    Flow::Yield
                }
                _ => Flow::Done,
            },
            _ => unreachable!("behavior routed to the wrong step module"),
        }
    }
}
