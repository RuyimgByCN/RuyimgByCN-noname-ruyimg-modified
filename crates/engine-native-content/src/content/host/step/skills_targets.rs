use super::super::*;

/// 写轮眼（欢乐卡牌/card/skill.js:107）可窃技能过滤：对齐 `target.getSkills(null, false, false)`，
/// 仅排除装备技（`equip_skills`）、`charlotte`/全局规则技，及非窃技残余的失效 `异殇` 印记；
/// 目标的禁用技能（`enabled == false`）与使用者已拥有的技能均允许列出与窃取。
pub(in crate::content::host) fn xuelunyang_stealable_skills(
    state: &crate::state::GameStateCore,
    catalog: &crate::catalog::Catalog,
    _owner: PlayerId,
    target: PlayerId,
) -> Vec<crate::ids::SkillDefId> {
    let equips = &state.player(target).equips;
    state
        .skills
        .range((target, 0)..(target, u16::MAX))
        .filter(|((_, def), inst)| {
            if !inst.enabled && catalog.skill_name(*def) == crate::vocabulary::names::YI_SHANG {
                return false;
            }
            if catalog.is_charlotte_skill(*def) {
                return false;
            }
            let skill_name = catalog.skills[*def as usize].name;
            !equips
                .iter()
                .any(|c| catalog.cards[*c as usize].equip_skills.contains(&skill_name))
        })
        .map(|((_, def), _)| *def)
        .collect()
}

pub(in crate::content::host) fn xuelunyang_targets(
    state: &crate::state::GameStateCore,
    catalog: &crate::catalog::Catalog,
    owner: PlayerId,
) -> Vec<PlayerId> {
    (0..state.players.len())
        .map(|i| i as PlayerId)
        .filter(|&i| {
            i != owner
                && state.player(i).alive
                && !xuelunyang_stealable_skills(state, catalog, owner, i).is_empty()
        })
        .collect()
}

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
                // 写轮眼（欢乐卡牌/card/skill.js:107）多步：确认后 ① 选目标（其他有可窃技能者）→
                // ② 选其一个技能（Tag=列表下标，可取消，answer 时重枚举映射）→
                // 临时授予至回合结束/phaseAfter 前。
                match self.stage {
                    HostStage::Act => {
                        let candidates: Vec<ChoiceOption> =
                            xuelunyang_targets(ctx.core, ctx.catalog, self.owner)
                                .into_iter()
                                .map(ChoiceOption::Player)
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
                                allow_cancel: true,
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
                        let owned = xuelunyang_stealable_skills(
                            ctx.core,
                            ctx.catalog,
                            self.owner,
                            target,
                        );
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
                                allow_cancel: true,
                            },
                        }
                    }
                    HostStage::ComparePlayerPick => {
                        let Some(ChoiceOption::Tag(idx)) = ctx.answer().first().copied() else {
                            return Flow::Done;
                        };
                        let Some(target) = self.target else {
                            return Flow::Done;
                        };
                        let owned = xuelunyang_stealable_skills(
                            ctx.core,
                            ctx.catalog,
                            self.owner,
                            target,
                        );
                        let Some(def) = owned.get(idx as usize).copied() else {
                            return Flow::Done;
                        };
                        if ctx.catalog.skill_name(def) == crate::vocabulary::names::YI_SHANG
                            && !ctx.core.skills.contains_key(&(self.owner, def))
                        {
                            ctx.core.vm_aux.flags.insert(
                                (self.owner, format!("xuelunyang_temp:{def}")),
                                1,
                            );
                        }
                        ctx.add_skill_until_turn_end(self.owner, def);
                        Flow::Done
                    }
                    _ => Flow::Done,
                }
            }
            // 侠行（3D精选·character/skill.js:59-240）：
            // - 触发路径（self.card != Some(2)）：选择一张 he 牌置于牌堆顶（可取消），获得「侠」标记并视为事件角色
            //   使用一张【酒】（准备阶段开始）或【桃】（濒死时）；
            // - 主动转化路径（self.card == Some(2)，skill.js:163-208）：移去「侠」标记，0 卡视为使用【杀】。
            HostBehavior::DddXiaxing => {
                if self.card == Some(2) {
                    match self.stage {
                        HostStage::Act => {
                            let options: Vec<ChoiceOption> = ctx
                                .core
                                .alive_players()
                                .filter(|&p| {
                                    super::u01_fixes_1::is_sha_legal_play_target(
                                        ctx.core,
                                        ctx.catalog,
                                        self.owner,
                                        p,
                                    )
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
                                    prompt: "zhangba_sha_target",
                                    options,
                                    min: 1,
                                    max: 1,
                                    allow_cancel: true,
                                },
                            }
                        }
                        HostStage::TargetChoice => {
                            let Some(ChoiceOption::Player(target)) = ctx.answer().first().copied()
                            else {
                                return Flow::Done;
                            };
                            if !super::u01_fixes_1::is_sha_legal_play_target(
                                ctx.core,
                                ctx.catalog,
                                self.owner,
                                target,
                            ) {
                                return Flow::Done;
                            };
                            ctx.core
                                .vm_aux
                                .flags
                                .remove(&(self.owner, "dddxiaxing_xia".to_string()));
                            super::u01_fixes_1::consume_sha_play_phase(
                                ctx.core,
                                ctx.catalog,
                                self.owner,
                            );
                            ctx.log(LogEvent::SkillTriggered {
                                skill: "侠行",
                                owner: self.owner,
                                presentation: None,
                            });
                            ctx.emit(EventData::CardUsed(crate::event::CardUseData {
                                card: None,
                                player: self.owner,
                                targets: vec![target],
                                nature: crate::event::DamageNature::Normal,
                                use_as: Some(CardKind::Sha),
                                cancelled: false,
                            }));
                            Flow::Done
                        }
                        _ => Flow::Done,
                    }
                } else {
                    match self.stage {
                        HostStage::Act => {
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
                                return if self.card == Some(1) {
                                    super::u01_fixes_1::finish_respond_park_pub(ctx, self.owner, false)
                                } else {
                                    Flow::Done
                                };
                            }
                            self.stage = HostStage::SelectCard;
                            Flow::WaitChoice {
                                observer: self.owner,
                                spec: ChoiceSpec {
                                    kind: ChoiceKind::SelectCards,
                                    prompt: "dddxiaxing_card",
                                    options,
                                    min: 1,
                                    max: 1,
                                    allow_cancel: true,
                                },
                            }
                        }
                        HostStage::SelectCard => {
                            let Some(ChoiceOption::Card(picked)) = ctx.answer().first().copied()
                            else {
                                return if self.card == Some(1) {
                                    super::u01_fixes_1::finish_respond_park_pub(ctx, self.owner, false)
                                } else {
                                    Flow::Done
                                };
                            };
                            if !ctx.core.player(self.owner).hand.contains(&picked)
                                && !ctx.core.player(self.owner).equips.contains(&picked)
                            {
                                return if self.card == Some(1) {
                                    super::u01_fixes_1::finish_respond_park_pub(ctx, self.owner, false)
                                } else {
                                    Flow::Done
                                };
                            }
                            ctx.take_owned_card(self.owner, picked);
                            ctx.core.deck.push(picked);
                            let round_tag = ctx.core.turn.round + 1000;
                            ctx.core
                                .vm_aux
                                .flags
                                .insert((self.owner, "dddxiaxing_round".to_string()), round_tag);
                            ctx.core
                                .vm_aux
                                .flags
                                .insert((self.owner, "dddxiaxing_xia".to_string()), 1);
                            ctx.log(LogEvent::SkillTriggered {
                                skill: "侠行",
                                owner: self.owner,
                                presentation: None,
                            });
                            self.stage = HostStage::DrawSettle;
                            Flow::Yield
                        }
                        HostStage::DrawSettle => {
                            if self.card == Some(1) {
                                return super::u01_fixes_1::finish_respond_park_pub(
                                    ctx, self.owner, true,
                                );
                            }
                            let beneficiary = self.target.unwrap_or(self.owner);
                            if ctx.core.player(beneficiary).alive {
                                ctx.emit(EventData::CardUsed(crate::event::CardUseData {
                                    card: None,
                                    player: beneficiary,
                                    targets: vec![beneficiary],
                                    nature: crate::event::DamageNature::Normal,
                                    use_as: Some(CardKind::Jiu),
                                    cancelled: false,
                                }));
                            }
                            Flow::Done
                        }
                        _ => Flow::Done,
                    }
                }
            }
            // 酒诗（L29 龙血玄黄·mobile.js:17251-17409，`mbcmjiushi`）
            HostBehavior::MbcmJiushi => match (self.card, self.stage) {
                // Part A: 出牌阶段主动发动（先 turnOver 再用虚拟【酒】）
                (Some(0), HostStage::Act) => {
                    let banned = ctx
                        .core
                        .vm_aux
                        .flags
                        .get(&(self.owner, "mbcmfangzhu_ban".to_string()))
                        .copied()
                        .unwrap_or(0)
                        > 0;
                    if ctx.core.player(self.owner).face_down || ctx.core.turn.jiu_used || banned {
                        return Flow::Done;
                    }
                    ctx.log(LogEvent::SkillTriggered {
                        skill: "mbcmjiushi",
                        owner: self.owner,
                        presentation: None,
                    });
                    // LIFO：先压虚拟【酒】CardUsed，再压 TurnOverBefore，确保翻面及获牌先完成
                    ctx.emit(EventData::CardUsed(crate::event::CardUseData {
                        card: None,
                        player: self.owner,
                        targets: vec![self.owner],
                        nature: crate::event::DamageNature::Normal,
                        use_as: Some(CardKind::Jiu),
                        cancelled: false,
                    }));
                    ctx.emit(EventData::TurnOverBefore {
                        player: self.owner,
                        cancelled: false,
                    });
                    Flow::Done
                }
                // Part B0: DamageBegin3 静默记录受伤前是否背面朝上
                (Some(3), HostStage::Act) => {
                    if ctx.current_damage().is_ok() && ctx.core.player(self.owner).face_down {
                        let root = ctx.current_event_frame_id();
                        ctx.core
                            .vm_aux
                            .flags
                            .insert((self.owner, format!("mbcmjiushi_dmg:{root}")), 1);
                    }
                    Flow::Done
                }
                // Part B1: DamageEnd 消费 DamageBegin3 记录后询问翻面
                (Some(1), HostStage::Act) => {
                    if ctx.current_damage().is_err() {
                        return Flow::Done;
                    }
                    let root = ctx.current_event_frame_id();
                    if ctx
                        .core
                        .vm_aux
                        .flags
                        .remove(&(self.owner, format!("mbcmjiushi_dmg:{root}")))
                        .is_none()
                    {
                        return Flow::Done;
                    }
                    self.stage = HostStage::TargetChoice;
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec::confirm(MBCMJIUSHI_TURNBACK_CONFIRM),
                    }
                }
                (Some(1), HostStage::TargetChoice) => {
                    if matches!(ctx.answer().first().copied(), Some(ChoiceOption::Confirm)) {
                        ctx.log(LogEvent::SkillTriggered {
                            skill: "mbcmjiushi",
                            owner: self.owner,
                            presentation: None,
                        });
                        ctx.emit(EventData::TurnOverBefore {
                            player: self.owner,
                            cancelled: false,
                        });
                    }
                    Flow::Done
                }
                // Part B2: 翻面后询问从牌堆（不含弃牌堆，`get.cardPile2`）获一张锦囊牌
                (Some(2), HostStage::Act) => {
                    self.stage = HostStage::SelectCard;
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec::confirm(MBCMJIUSHI_GAIN_CONFIRM),
                    }
                }
                (Some(2), HostStage::SelectCard) => {
                    if matches!(ctx.answer().first().copied(), Some(ChoiceOption::Confirm)) {
                        ctx.log(LogEvent::SkillTriggered {
                            skill: "mbcmjiushi",
                            owner: self.owner,
                            presentation: None,
                        });
                        if let Some(pos) = ctx
                            .core
                            .deck
                            .iter()
                            .rposition(|&c| ctx.catalog.card_kind(c).is_trick_kind())
                        {
                            let c = ctx.core.deck.remove(pos);
                            ctx.core.player_mut(self.owner).hand.push(c);
                            ctx.log(LogEvent::CardsDrawn {
                                player: self.owner,
                                cards: vec![c],
                            });
                            ctx.emit(EventData::CardsMoved {
                                from: self.owner,
                                to: self.owner,
                                cards: vec![c],
                            });
                        }
                    }
                    Flow::Done
                }
                _ => Flow::Done,
            },
            // 放逐（L29 龙血玄黄·mobile/skill.js:14559，`mbcmfangzhu`）
            HostBehavior::MbcmFangzhu => match self.stage {
                HostStage::Act => {
                    let last_t = ctx
                        .core
                        .vm_aux
                        .flags
                        .get(&(self.owner, "mbcmfangzhu_last_target".to_string()))
                        .copied();
                    let options: Vec<ChoiceOption> = ctx
                        .core
                        .alive_players()
                        .filter(|&p| p != self.owner && Some(u32::from(p)) != last_t)
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
                            prompt: MBCMFANGZHU_TARGET,
                            options,
                            min: 1,
                            max: 1,
                            allow_cancel: true,
                        },
                    }
                }
                HostStage::TargetChoice => {
                    let Some(ChoiceOption::Player(target)) = ctx.answer().first().copied() else {
                        if let Some(def) = ctx.catalog.skill_by_name(&self.def_key) {
                            if let Some(inst) = ctx.core.skills.get_mut(&(self.owner, def)) {
                                inst.once_used = false;
                            }
                        }
                        return Flow::Done;
                    };
                    let last_t = ctx
                        .core
                        .vm_aux
                        .flags
                        .get(&(self.owner, "mbcmfangzhu_last_target".to_string()))
                        .copied();
                    if !ctx.core.player(target).alive
                        || target == self.owner
                        || Some(u32::from(target)) == last_t
                    {
                        return Flow::Done;
                    }
                    self.target = Some(target);
                    self.stage = HostStage::SelectNature;
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::PickTag,
                            prompt: MBCMFANGZHU_OPTION,
                            options: vec![ChoiceOption::Tag(0), ChoiceOption::Tag(1)],
                            min: 1,
                            max: 1,
                            allow_cancel: false,
                        },
                    }
                }
                HostStage::SelectNature => {
                    let Some(target) = self.target else {
                        return Flow::Done;
                    };
                    let tag = match ctx.answer().first().copied() {
                        Some(ChoiceOption::Tag(t)) => t,
                        _ => 0,
                    };
                    ctx.core
                        .vm_aux
                        .flags
                        .insert((self.owner, "mbcmfangzhu_last_target".to_string()), u32::from(target));
                    ctx.core
                        .vm_aux
                        .flags
                        .insert((self.owner, "mbcmfangzhu_used_in_play".to_string()), 1);
                    if tag == 0 {
                        ctx.core
                            .vm_aux
                            .flags
                            .insert((target, "mbcmfangzhu_ban".to_string()), 1);
                    } else {
                        ctx.core
                            .vm_aux
                            .flags
                            .insert((target, "mbcmfangzhu_baiban".to_string()), 1);
                        let defs_to_disable: Vec<u16> = ctx
                            .core
                            .skills
                            .iter()
                            .filter_map(|(&(p, def), inst)| {
                                let exempt = ctx.catalog.is_forever_skill(def)
                                    || ctx.catalog.is_charlotte_skill(def);
                                (p == target && inst.enabled && !exempt).then_some(def)
                            })
                            .collect();
                        for def in defs_to_disable {
                            if let Some(inst) = ctx.core.skills.get_mut(&(target, def)) {
                                inst.enabled = false;
                            }
                            ctx.core
                                .vm_aux
                                .flags
                                .insert((target, format!("mbcmfangzhu_disabled:{def}")), 1);
                        }
                    }
                    ctx.log(LogEvent::SkillTriggered {
                        skill: "mbcmfangzhu",
                        owner: self.owner,
                        presentation: None,
                    });
                    Flow::Done
                }
                _ => Flow::Done,
            },
            // 决进（L29 龙血玄黄·mobile/skill.js:14751，`mbjuejin`）
            HostBehavior::MbJuejin => {
                if ctx
                    .core
                    .vm_aux
                    .flags
                    .get(&(self.owner, "mbjuejin_used".to_string()))
                    .copied()
                    .unwrap_or(0)
                    > 0
                {
                    return Flow::Done;
                }
                ctx.core
                    .vm_aux
                    .flags
                    .insert((self.owner, "mbjuejin_used".to_string()), 1);
                ctx.core
                    .vm_aux
                    .flags
                    .insert((0, "mbjuejin_xiangsicunwei".to_string()), 1);
                ctx.core
                    .vm_aux
                    .flags
                    .insert((self.owner, "mbjuejin_xiangsicunwei".to_string()), 1);
                for p in ctx.core.seat_order_from(self.owner) {
                    let old_hp = ctx.core.player(p).hp;
                    let plus = if p == self.owner { 2u8 } else { 0u8 };
                    if old_hp > 1 {
                        let delt = (old_hp - 1) as u8;
                        ctx.core.player_mut(p).hp = 1;
                        ctx.core.player_mut(p).hujia =
                            ctx.core.player(p).hujia.saturating_add(delt + plus);
                    } else if old_hp < 1 {
                        let delt = (1 - old_hp) as u8;
                        ctx.core.player_mut(p).hp = 1;
                        ctx.core.player_mut(p).hujia =
                            ctx.core.player(p).hujia.saturating_add(delt + plus);
                    } else if plus > 0 {
                        ctx.core.player_mut(p).hujia =
                            ctx.core.player(p).hujia.saturating_add(plus);
                    }
                }
                let is_stj = |c: CardId, cat: &crate::catalog::Catalog| {
                    matches!(
                        cat.card_kind(c),
                        CardKind::Shan | CardKind::Tao | CardKind::Jiu
                    )
                };
                for p in ctx.core.seat_order_from(self.owner) {
                    let hand_stj: Vec<CardId> = ctx
                        .core
                        .player(p)
                        .hand
                        .iter()
                        .copied()
                        .filter(|&c| is_stj(c, ctx.catalog))
                        .collect();
                    if hand_stj.is_empty() {
                        continue;
                    }
                    for &c in &hand_stj {
                        ctx.take_hand_card(p, c);
                        if !ctx.core.detached_cards.iter().any(|(dc, _)| *dc == c) {
                            ctx.core
                                .detached_cards
                                .push((c, crate::event::DetachedKind::Destroyed));
                        }
                    }
                    ctx.note_cards_lost(p, &hand_stj);
                    ctx.emit(EventData::CardsLost {
                        player: p,
                        origins: vec![crate::event::LostOrigin::Hand; hand_stj.len()],
                        cards: hand_stj,
                    });
                }
                ctx.log(LogEvent::SkillTriggered {
                    skill: "mbjuejin",
                    owner: self.owner,
                    presentation: None,
                });
                Flow::Done
            }
            _ => unreachable!("behavior routed to the wrong step module"),
        }
    }
}
