use super::super::*;

impl HostMachine {
    pub(super) fn step_skills_targets(&mut self, ctx: &mut Ctx<'_>) -> Flow {
        match self.behavior {
            // 门神：选一名其他角色 → 授予门神标记
            HostBehavior::YxsMenshen => match self.stage {
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
                            prompt: "yxs_menshen_target",
                            options,
                            min: 1,
                            max: 1,
                            allow_cancel: true,
                        },
                    }
                }
                HostStage::TargetChoice => {
                    let Some(ChoiceOption::Player(target)) = ctx.answer().first().copied() else {
                        return Flow::Done;
                    };
                    if !ctx.core.player(target).alive || target == self.owner {
                        return Flow::Done;
                    }
                    let Some(def) = ctx.catalog.skill_by_name(U08_YXS_MENSHEN2_KEY) else {
                        return Flow::Done;
                    };
                    ctx.insert_skill(
                        (target, def),
                        crate::state::SkillInstance {
                            owner: target,
                            enabled: true,
                            once_used: false,
                            expires_end_of_turn: false,
                            rounds: 0,
                        },
                    );
                    ctx.core
                        .vm_aux
                        .marks
                        .insert((target, u32::from(def)), i32::from(self.owner));
                    ctx.log(LogEvent::SkillTriggered {
                        skill: "门神",
                        owner: self.owner, presentation: None });
                    Flow::Done
                }
                _ => Flow::Done,
            },
            // 李香龙：选攻击范围内目标 → 使用虚拟杀
            HostBehavior::Lxianglong => match self.stage {
                HostStage::Act => {
                    let range = ctx.core.attack_range(ctx.catalog, self.owner);
                    let options: Vec<ChoiceOption> = ctx
                        .core
                        .alive_players()
                        .filter(|p| {
                            *p != self.owner
                                && ctx.core.computed_distance(ctx.catalog, self.owner, *p) <= range
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
                            prompt: "lxianglong_target",
                            options,
                            min: 1,
                            max: 1,
                            allow_cancel: true,
                        },
                    }
                }
                HostStage::TargetChoice => {
                    let Some(ChoiceOption::Player(target)) = ctx.answer().first().copied() else {
                        return Flow::Done;
                    };
                    if !ctx.core.player(target).alive {
                        return Flow::Done;
                    }
                    ctx.log(LogEvent::SkillTriggered {
                        skill: "李香龙",
                        owner: self.owner, presentation: None });
                    ctx.use_virtual_card(self.owner, crate::catalog::CardKind::Sha, vec![target]);
                    self.stage = HostStage::ShuchuiTargetLoop;
                    Flow::Yield
                }
                _ => Flow::Done,
            },
            // 冰封：翻面自己与至多 3 名目标 + 失去体力上限 + 授予封锁
            HostBehavior::Bingfeng => match self.stage {
                HostStage::Act => {
                    let options: Vec<ChoiceOption> = ctx
                        .core
                        .alive_players()
                        .filter(|p| *p != self.owner && !ctx.core.player(*p).face_down)
                        .map(ChoiceOption::Player)
                        .collect();
                    if options.is_empty() {
                        return Flow::Done;
                    }
                    self.stage = HostStage::TargetChoice;
                    let max = options.len().min(3);
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectPlayers,
                            prompt: "bingfeng_targets",
                            options,
                            min: 1,
                            max,
                            allow_cancel: false,
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
                        return Flow::Done;
                    }
                    // 觉醒（一次性）：禁用自身实例
                    if let Some(def) = ctx.catalog.skill_by_name(SWD_BINGFENG_KEY) {
                        if let Some(inst) = ctx.core.skills.get_mut(&(self.owner, def)) {
                            inst.enabled = false;
                        }
                        ctx.log(LogEvent::SkillToggled {
                            player: self.owner,
                            skill: ctx.catalog.skill_name(def),
                            enabled: false,
                        });
                    }
                    // 移除玄咒（上游 removeSkill('xuanzhou')）
                    if let Some(def) = ctx.catalog.skill_by_name("xuanzhou") {
                        if ctx.core.skills.remove(&(self.owner, def)).is_some() {
                            ctx.log(LogEvent::SkillToggled {
                                player: self.owner,
                                skill: ctx.catalog.skill_name(def),
                                enabled: false,
                            });
                        }
                    }
                    // 失去 1 点体力上限
                    ctx.lose_max_hp(self.owner, 1);
                    // 翻面自己 + 各目标（TurnOverBefore 提交帧；取消技照常生效）
                    ctx.emit(EventData::TurnOverBefore {
                        player: self.owner,
                        cancelled: false,
                    });
                    for t in targets.iter().copied() {
                        if ctx.core.player(t).alive && !ctx.core.player(t).face_down {
                            ctx.emit(EventData::TurnOverBefore {
                                player: t,
                                cancelled: false,
                            });
                        }
                    }
                    // 授予「冰封·封」（自己与各目标）
                    let mut holders = vec![self.owner];
                    holders.extend(targets.iter().copied());
                    if let Some(def) = ctx.catalog.skill_by_name(SWD_BINGFENG2_KEY) {
                        for h in holders {
                            if !ctx.core.skills.contains_key(&(h, def)) {
                                ctx.insert_skill(
                                    (h, def),
                                    crate::state::SkillInstance {
                                        owner: h,
                                        enabled: true,
                                        once_used: false,
                                        expires_end_of_turn: false,
                                        rounds: 0,
                                    },
                                );
                                ctx.log(LogEvent::SkillToggled {
                                    player: h,
                                    skill: ctx.catalog.skill_name(def),
                                    enabled: true,
                                });
                            }
                        }
                    }
                    ctx.log(LogEvent::SkillTriggered {
                        skill: "冰封",
                        owner: self.owner, presentation: None });
                    Flow::Done
                }
                _ => Flow::Done,
            },
            HostBehavior::HuanleXuelunyangCard => {
                // 写轮眼（E03）多步：确认后 ① 选目标（其他有技能者）→
                // ② 选其一个技能（Tag=列表下标，answer 时重枚举映射）→
                // 临时授予至回合结束。
                match self.stage {
                    HostStage::Act => {
                        let candidates: Vec<ChoiceOption> = (0..ctx.core.players.len())
                            .filter(|&i| {
                                i as PlayerId != self.owner
                                    && ctx.core.player(i as PlayerId).alive
                                    && ctx.core.skills.keys().any(|(o, _)| *o == i as PlayerId)
                            })
                            .map(|i| ChoiceOption::Player(i as PlayerId))
                            .collect();
                        if candidates.is_empty() {
                            return Flow::Done;
                        }
                        self.stage = HostStage::CompareTargetPick;
                        Flow::WaitChoice {
                            observer: self.owner,
                            spec: ChoiceSpec {
                                kind: ChoiceKind::SelectPlayers,
                                prompt: "huanle_xuelunyang_target",
                                options: candidates,
                                min: 1,
                                max: 1,
                                allow_cancel: false,
                            },
                        }
                    }
                    HostStage::CompareTargetPick => {
                        // 从 SelectPlayers 应答填充窃取目标。
                        let ChoiceOption::Player(target) =
                            ctx.answer().first().copied().unwrap_or(ChoiceOption::Pass)
                        else {
                            return Flow::Done;
                        };
                        self.target = Some(target);
                        // 目标的可窃技能（目录注册且非本窃取技能自身），
                        // 按目录序确定性枚举。
                        let owned: Vec<&str> = ctx
                            .core
                            .skills
                            .range((target, 0)..(target, u16::MAX))
                            .map(|((_, d), _)| ctx.catalog.skills[*d as usize].name)
                            .collect();
                        let tags: Vec<ChoiceOption> = (0..owned.len())
                            .map(|i| ChoiceOption::Tag(i as u8))
                            .collect();
                        if tags.is_empty() {
                            return Flow::Done;
                        }
                        self.stage = HostStage::ComparePlayerPick;
                        // 记录选项数供应答映射：借助 cards 字段存选项数。
                        self.cards.clear();
                        self.cards.push(owned.len() as CardId);
                        Flow::WaitChoice {
                            observer: self.owner,
                            spec: ChoiceSpec {
                                kind: ChoiceKind::PickTag,
                                prompt: "huanle_xuelunyang_skill",
                                options: tags,
                                min: 1,
                                max: 1,
                                allow_cancel: false,
                            },
                        }
                    }
                    HostStage::ComparePlayerPick => {
                        let picked = ctx.answer().first().copied().expect("PickTag 应答存在");
                        let ChoiceOption::Tag(idx) = picked else {
                            return Flow::Done;
                        };
                        let target = self.target.expect("已选目标");
                        let owned: Vec<crate::ids::SkillDefId> = ctx
                            .core
                            .skills
                            .range((target, 0)..(target, u16::MAX))
                            .map(|((_, d), _)| *d)
                            .collect();
                        let Some(def) = owned.get(idx as usize).copied() else {
                            return Flow::Done;
                        };
                        ctx.add_skill_until_turn_end(self.owner, def);
                        Flow::Done
                    }
                    _ => Flow::Done,
                }
            }
            _ => unreachable!("behavior routed to the wrong step module"),
        }
    }
}
