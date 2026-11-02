use super::super::*;

impl HostMachine {
    pub(super) fn step_xianjian_targets(&mut self, ctx: &mut Ctx<'_>) -> Flow {
        match self.behavior {
            HostBehavior::XianjianYuexing => match self.stage {
                // 月行（U05 xianjian 龙幽，规则 v159）：指定一名其他角色
                // → 双向互指距离基数交换，自己出牌阶段结束解除（commit_
                // phase Play 尾钩；computed_distance 座位基数代换）。
                HostStage::Act => {
                    let options: Vec<ChoiceOption> = ctx
                        .core
                        .alive_players()
                        .filter(|p| *p != self.owner)
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
                            prompt: "wudian_xj_yuexing_target",
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
                        if t != self.owner && ctx.core.player(t).alive {
                            ctx.core.player_mut(self.owner).yuexing_pair = Some(t);
                            ctx.core.player_mut(t).yuexing_pair = Some(self.owner);
                        }
                    }
                    Flow::Done
                }
                _ => Flow::Done,
            },
            HostBehavior::XianjianXjzhimeng => match self.stage {
                // 织梦（U05 xianjian 柳梦璃，规则 v167）：阶段末选一名其
                // 他角色→牌堆顶一张暗置其武将牌上（off_court）+runtime
                // 授织梦·梦实例（上游 addSkill 同口径）。取消=放弃。
                // 首臂发问。
                HostStage::Act => {
                    let options: Vec<ChoiceOption> = ctx
                        .core
                        .alive_players()
                        .filter(|p| *p != self.owner)
                        .map(ChoiceOption::Player)
                        .collect();
                    if options.is_empty() || ctx.core.deck.is_empty() {
                        return Flow::Done;
                    }
                    self.stage = HostStage::TargetChoice;
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectPlayers,
                            prompt: WUDIAN_XJ_ZHIMENG_TARGET,
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
                        if t != self.owner && ctx.core.player(t).alive {
                            if let Some(c) = ctx.core.deck.pop() {
                                ctx.core.player_mut(t).off_court.push(c);
                            }
                            if let Some(def) = ctx.catalog.skill_by_name("xjzhimeng2") {
                                ctx.insert_skill(
                                    (t, def),
                                    crate::state::SkillInstance {
                                        owner: t,
                                        enabled: true,
                                        once_used: false,
                                        expires_end_of_turn: false,
                                        rounds: 0,
                                    },
                                );
                            }
                        }
                    }
                    Flow::Done
                }
                _ => Flow::Done,
            },
            HostBehavior::XianjianHusha => match self.stage {
                // 虎煞（U05 xianjian 王小虎，规则 v169）：阶段末模式选择
                //（Tag0=1 枚虚拟杀+选目标；Tag1=2 枚虚拟南蛮 AOE；取消=
                // 放弃；元素毁灭牌种不存在登记不转）→ 虚拟用牌原语
                //（use_virtual_card 首例：牌堆底载体+use_as 声明）。
                HostStage::Act => {
                    let mark = ctx.core.player(self.owner).husha_mark;
                    let mut options: Vec<ChoiceOption> = Vec::new();
                    let has_sha_target = ctx.core.alive_players().any(|p| p != self.owner);
                    if mark >= 1 && has_sha_target {
                        options.push(ChoiceOption::Tag(0));
                    }
                    if mark >= 2 {
                        options.push(ChoiceOption::Tag(1));
                    }
                    if options.is_empty() {
                        return Flow::Done;
                    }
                    self.stage = HostStage::TargetChoice;
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::PickTag,
                            prompt: WUDIAN_XJ_HUSHA_MODE,
                            options,
                            min: 1,
                            max: 1,
                            allow_cancel: true,
                        },
                    }
                }
                HostStage::TargetChoice => {
                    let mode = match ctx.answer().first() {
                        Some(ChoiceOption::Tag(t)) => *t,
                        _ => {
                            return Flow::Done; // 取消=放弃
                        }
                    };
                    match mode {
                        0 => {
                            // 虚拟杀：扣 1 枚+选目标
                            ctx.core.player_mut(self.owner).husha_mark -= 1;
                            let options: Vec<ChoiceOption> = ctx
                                .core
                                .alive_players()
                                .filter(|p| *p != self.owner)
                                .map(ChoiceOption::Player)
                                .collect();
                            if options.is_empty() {
                                return Flow::Done;
                            }
                            self.stage = HostStage::SelectNature; // 复用为出杀目标段
                            Flow::WaitChoice {
                                observer: self.owner,
                                spec: ChoiceSpec {
                                    kind: ChoiceKind::SelectPlayers,
                                    prompt: WUDIAN_XJ_HUSHA_TARGET,
                                    options,
                                    min: 1,
                                    max: 1,
                                    allow_cancel: false,
                                },
                            }
                        }
                        1 => {
                            // 虚拟南蛮：扣 2 枚+AOE（全其他存活）
                            ctx.core.player_mut(self.owner).husha_mark -= 2;
                            let targets: Vec<PlayerId> = ctx
                                .core
                                .seat_order_from(self.owner)
                                .into_iter()
                                .filter(|p| *p != self.owner)
                                .collect();
                            ctx.use_virtual_card(self.owner, CardKind::SavageAttack, targets);
                            Flow::Done
                        }
                        _ => Flow::Done,
                    }
                }
                // 复用段：虚拟杀目标选择应答 → use_virtual_card(Sha)
                HostStage::SelectNature => {
                    if let Some(ChoiceOption::Player(t)) = ctx.answer().first() {
                        let t = *t;
                        if t != self.owner && ctx.core.player(t).alive {
                            ctx.use_virtual_card(self.owner, CardKind::Sha, vec![t]);
                        }
                    }
                    Flow::Done
                }
                _ => Flow::Done,
            },
            HostBehavior::XianjianChangnian => match self.stage {
                // 长念（U05 xianjian 柳梦璃，规则 v170）：死亡前置窗口遗
                // 言——选一名其他角色移交 hej 全部牌（全部入手）+runtime
                // 授长念·思实例；随后 Commit 正常死亡。首臂发问。
                HostStage::Act => {
                    let options: Vec<ChoiceOption> = ctx
                        .core
                        .alive_players()
                        .filter(|p| *p != self.owner)
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
                            prompt: WUDIAN_XJ_CHANGNIAN_TARGET,
                            options,
                            min: 1,
                            max: 1,
                            allow_cancel: false,
                        },
                    }
                }
                HostStage::TargetChoice => {
                    let Some(ChoiceOption::Player(t)) = ctx.answer().first() else {
                        return Flow::Done;
                    };
                    let t = *t;
                    if t == self.owner || !ctx.core.player(t).alive {
                        return Flow::Done;
                    }
                    // 移交 hej 全部牌（上游 gain 全部入手同口径；装备撤
                    // 技后入手）
                    let equips: Vec<CardId> = ctx.core.player(self.owner).equips.clone();
                    for c in equips {
                        ctx.core.player_mut(self.owner).equips.retain(|x| *x != c);
                        crate::kernel::revoke_equip_skills(ctx.core, ctx.catalog, self.owner, c);
                        ctx.core.player_mut(t).hand.push(c);
                    }
                    let judge: Vec<CardId> = ctx.core.player(self.owner).judge.clone();
                    for c in judge {
                        ctx.core.player_mut(self.owner).judge.retain(|x| *x != c);
                        ctx.core.player_mut(t).hand.push(c);
                    }
                    let hand: Vec<CardId> = ctx.core.player(self.owner).hand.clone();
                    for c in hand {
                        ctx.take_hand_card(self.owner, c);
                        ctx.core.player_mut(t).hand.push(c);
                    }
                    if let Some(def) = ctx.catalog.skill_by_name("changnian2") {
                        ctx.insert_skill(
                            (t, def),
                            crate::state::SkillInstance {
                                owner: t,
                                enabled: true,
                                once_used: false,
                                expires_end_of_turn: false,
                                rounds: 0,
                            },
                        );
                    }
                    ctx.log(LogEvent::CardsGiven {
                        from: self.owner,
                        to: t,
                        cards: {
                            let mut all = Vec::new();
                            all.extend(ctx.core.player(t).hand.iter().copied());
                            all
                        },
                    });
                    Flow::Done
                }
                _ => Flow::Done,
            },
            HostBehavior::XianjianFenglue => match self.stage {
                // 凤略（U05 xianjian 罗招嫣，规则 v177）：出牌阶段开始选
                // 一名其他角色→逐张自动对其使用手中可指定其的杀与决斗
                //（真实牌 take 后 emit CardUsed=虚拟管线复用真实牌变体；
                // **登记差异**：其余锦囊不自动使用）。首臂发问，取消=放
                // 弃；逐张 Yield。
                HostStage::Act => {
                    let options: Vec<ChoiceOption> = ctx
                        .core
                        .alive_players()
                        .filter(|p| *p != self.owner && ctx.core.player(*p).alive)
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
                            prompt: WUDIAN_XJ_FENGLUE_TARGET,
                            options,
                            min: 1,
                            max: 1,
                            allow_cancel: true,
                        },
                    }
                }
                HostStage::TargetChoice => {
                    if self.target.is_none() {
                        if let Some(ChoiceOption::Player(t)) = ctx.answer().first().cloned() {
                            if t != self.owner && ctx.core.player(t).alive {
                                self.target = Some(t);
                            }
                        }
                        if self.target.is_none() {
                            return Flow::Done;
                        }
                        // 收集可对其使用的杀（距离内）与决斗
                        let t = self.target.expect("checked");
                        let usable: Vec<CardId> = ctx
                            .core
                            .player(self.owner)
                            .hand
                            .iter()
                            .copied()
                            .filter(|c| {
                                let k = ctx.catalog.card_kind(*c);
                                match k {
                                    CardKind::Sha => {
                                        ctx.core.computed_distance(ctx.catalog, self.owner, t)
                                            <= ctx.core.attack_range(ctx.catalog, self.owner)
                                    }
                                    CardKind::Duel => true,
                                    _ => false,
                                }
                            })
                            .collect();
                        self.cards = usable;
                    }
                    // 逐张使用（emit CardUsed 由提交阶段 take 真实牌）
                    let Some(c) = self.cards.first().copied() else {
                        return Flow::Done;
                    };
                    self.cards.remove(0);
                    let t = self.target.expect("checked");
                    ctx.emit(EventData::CardUsed(crate::event::CardUseData {
                        card: Some(c),
                        player: self.owner,
                        targets: vec![t],
                        nature: crate::event::DamageNature::Normal,
                        use_as: None,
                        cancelled: false,
                    }));
                    Flow::Yield
                }
                _ => Flow::Done,
            },
            _ => unreachable!("behavior routed to the wrong step module"),
        }
    }
}
