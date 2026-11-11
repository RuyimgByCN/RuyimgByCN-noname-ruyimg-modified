use super::super::*;

impl HostMachine {
    pub(super) fn step_boss(&mut self, ctx: &mut Ctx<'_>) -> Flow {
        match self.behavior {
            // 青木：标记四象链 + 表现隐藏（座位交互门控近似=日志；无存档字段）
            HostBehavior::BossQingmu2 => {
                sixiang_mark_hide(ctx, self.owner, "青木", false);
                Flow::Done
            }
            // 青木·转阶：取消死亡执行 + 换勾芒 + 全场回满 + 两书精 + 挑战者重整
            HostBehavior::BossQingmu2x => {
                sixiang_run_chain(
                    ctx,
                    self.owner,
                    &SixiangChainParams {
                        next_boss: "勾芒",
                        add_fellows: &["书精", "书精"],
                        side_reinits: &[],
                        heal_all: true,
                        chain_level: 2,
                    },
                );
                Flow::Done
            }
            // 青木3：三人隐藏（本体+环邻）
            HostBehavior::BossQingmu3 => {
                sixiang_mark_hide(ctx, self.owner, "青木3", true);
                Flow::Done
            }
            // 青木3·转阶：太昊 + 侧座勾芒/书精 + 挑战者重整
            HostBehavior::BossQingmu3x => {
                sixiang_run_chain(
                    ctx,
                    self.owner,
                    &SixiangChainParams {
                        next_boss: "太昊",
                        add_fellows: &[],
                        side_reinits: &[(-1, "勾芒"), (1, "书精")],
                        heal_all: false,
                        chain_level: 3,
                    },
                );
                Flow::Done
            }
            // 白芒 / 赤炎 / 玄麟：同青木 dieBegin hide（D4/B2）
            HostBehavior::BossBaimang2 => {
                sixiang_mark_hide(ctx, self.owner, "白芒", false);
                Flow::Done
            }
            HostBehavior::BossChiyan2 => {
                sixiang_mark_hide(ctx, self.owner, "赤炎", false);
                Flow::Done
            }
            HostBehavior::BossXuanlin2 => {
                sixiang_mark_hide(ctx, self.owner, "玄麟", false);
                Flow::Done
            }
            // 白芒·转阶：蓐收 + 两明星 + 全场回满 + 挑战者重整
            HostBehavior::BossBaimang2x => {
                sixiang_run_chain(
                    ctx,
                    self.owner,
                    &SixiangChainParams {
                        next_boss: "蓐收",
                        add_fellows: &["明星", "明星"],
                        side_reinits: &[],
                        heal_all: true,
                        chain_level: 2,
                    },
                );
                Flow::Done
            }
            // 赤炎·转阶：祝融 + 两焰灵 + 全场回满 + 挑战者重整
            HostBehavior::BossChiyan2x => {
                sixiang_run_chain(
                    ctx,
                    self.owner,
                    &SixiangChainParams {
                        next_boss: "祝融",
                        add_fellows: &["焰灵", "焰灵"],
                        side_reinits: &[],
                        heal_all: true,
                        chain_level: 2,
                    },
                );
                Flow::Done
            }
            // 玄麟·转阶：共工 + 一玄冥 + 全场回满 + 挑战者重整
            HostBehavior::BossXuanlin2x => {
                sixiang_run_chain(
                    ctx,
                    self.owner,
                    &SixiangChainParams {
                        next_boss: "共工",
                        add_fellows: &["玄冥"],
                        side_reinits: &[],
                        heal_all: true,
                        chain_level: 2,
                    },
                );
                Flow::Done
            }
            // 白芒3 / 赤炎3 / 玄麟3：同青木3 三人 hide（D4/B3）
            HostBehavior::BossBaimang3 => {
                sixiang_mark_hide(ctx, self.owner, "白芒3", true);
                Flow::Done
            }
            HostBehavior::BossChiyan3 => {
                sixiang_mark_hide(ctx, self.owner, "赤炎3", true);
                Flow::Done
            }
            HostBehavior::BossXuanlin3 => {
                sixiang_mark_hide(ctx, self.owner, "玄麟3", true);
                Flow::Done
            }
            // 白芒3·转阶：少昊 + 侧座蓐收/明星 + 一明星 fellow
            HostBehavior::BossBaimang3x => {
                sixiang_run_chain(
                    ctx,
                    self.owner,
                    &SixiangChainParams {
                        next_boss: "少昊",
                        add_fellows: &["明星"],
                        side_reinits: &[(-1, "蓐收"), (1, "明星")],
                        heal_all: false,
                        chain_level: 3,
                    },
                );
                Flow::Done
            }
            // 赤炎3·转阶：炎帝 + 侧座祝融/焰灵
            HostBehavior::BossChiyan3x => {
                sixiang_run_chain(
                    ctx,
                    self.owner,
                    &SixiangChainParams {
                        next_boss: "炎帝",
                        add_fellows: &[],
                        side_reinits: &[(-1, "祝融"), (1, "焰灵")],
                        heal_all: false,
                        chain_level: 3,
                    },
                );
                Flow::Done
            }
            // 玄麟3·转阶：颛顼 + 共工 fellow + 次座玄冥
            HostBehavior::BossXuanlin3x => {
                sixiang_run_chain(
                    ctx,
                    self.owner,
                    &SixiangChainParams {
                        next_boss: "颛顼",
                        add_fellows: &["共工"],
                        side_reinits: &[(1, "玄冥")],
                        heal_all: false,
                        chain_level: 3,
                    },
                );
                Flow::Done
            }
            // 后援3/4：死亡前置隐藏（日志近似；不改 stage.kind）
            HostBehavior::BossBianshen3 => {
                bianshen_mark_hide(ctx, self.owner, "后援3");
                Flow::Done
            }
            HostBehavior::BossBianshen4 => {
                bianshen_mark_hide(ctx, self.owner, "后援4");
                Flow::Done
            }
            // 后援3·转阶：种子化 → 白无常/黑无常
            HostBehavior::BossBianshen3x => {
                bianshen_run_change_boss(ctx, self.owner, &["白无常", "黑无常"]);
                Flow::Done
            }
            // 后援4·转阶：种子化 → 夜叉/罗刹
            HostBehavior::BossBianshen4x => {
                bianshen_run_change_boss(ctx, self.owner, &["夜叉", "罗刹"]);
                Flow::Done
            }
            HostBehavior::BossHuanren => {
                huanren_on_game_start(ctx, self.owner);
                Flow::Done
            }
            HostBehavior::BossHuanrenDie => match self.stage {
                HostStage::Act => huanren_die_ask(ctx, self),
                HostStage::HuanrenPick => {
                    huanren_die_commit(ctx, self);
                    Flow::Done
                }
                _ => Flow::Done,
            },
            HostBehavior::BossQingyi => {
                wing_qing(ctx, self.owner);
                Flow::Done
            }
            HostBehavior::BossChiyi => {
                wing_chi(ctx, self.owner);
                Flow::Done
            }
            HostBehavior::BossChiyiMark => {
                if !ctx.modify_damage_amount(1) { return Flow::Done; }
                Flow::Done
            }
            HostBehavior::BossBaiyi => {
                wing_bai_discard(ctx, self.owner);
                Flow::Done
            }
            HostBehavior::BossBaiyiThunder => {
                ctx.current_damage_mut().prevented = true;
                Flow::Done
            }
            // 穿云（太虚幻境·extension_skill.js:3806）：结束阶段开始，可选一名体力值 > 自己的其他角色造成 1 点伤害
            HostBehavior::BossChuanyun => match self.stage {
                HostStage::Act => {
                    let owner_hp = ctx.core.player(self.owner).hp;
                    let options: Vec<ChoiceOption> = ctx
                        .core
                        .alive_players()
                        .filter(|&p| p != self.owner && ctx.core.player(p).hp > owner_hp)
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
                            prompt: "boss_chuanyun_target",
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
                    let owner_hp = ctx.core.player(self.owner).hp;
                    if target == self.owner
                        || !ctx.core.player(target).alive
                        || ctx.core.player(target).hp <= owner_hp
                    {
                        return Flow::Done;
                    }
                    ctx.log(LogEvent::SkillTriggered {
                        skill: "穿云",
                        owner: self.owner,
                        presentation: None,
                    });
                    ctx.emit(EventData::Damage(crate::machine::damage_data(
                        Some(self.owner),
                        target,
                        1,
                    )));
                    Flow::Done
                }
                _ => Flow::Done,
            },
            // 天陨（太虚幻境·extension_skill.js:4085）：结束阶段开始，可选一名其他角色：
            // step 1 失去 1 点体力 → step 2 随机造成 2~3 点火焰伤害（Math.random()>0.4?2:3）→ step 3 弃置其所有装备牌。
            HostBehavior::BossTianyun => match self.stage {
                HostStage::Act => {
                    let options: Vec<ChoiceOption> = ctx
                        .core
                        .alive_players()
                        .filter(|&p| p != self.owner)
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
                            prompt: "boss_tianyun_target",
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
                    if target == self.owner || !ctx.core.player(target).alive {
                        return Flow::Done;
                    }
                    self.target = Some(target);
                    ctx.log(LogEvent::SkillTriggered {
                        skill: "天陨",
                        owner: self.owner,
                        presentation: None,
                    });
                    let dmg = if ctx.core.rng.gen_range(10) >= 4 {
                        2
                    } else {
                        3
                    };
                    // LIFO 压栈：先压 step 2 火焰伤害，再压 step 1 失去体力，使 step 1 先结算、step 2 次之；
                    // 让出至 ShuchuiTargetLoop 续行位待伤害帧弹栈后执行 step 3 弃置目标全部装备牌。
                    ctx.emit(EventData::Damage(crate::machine::damage_data_with(
                        Some(self.owner),
                        target,
                        dmg,
                        crate::event::DamageNature::Fire,
                        None,
                    )));
                    ctx.emit(EventData::LoseHp {
                        player: self.owner,
                        amount: 1,
                    });
                    self.stage = HostStage::ShuchuiTargetLoop;
                    Flow::Yield
                }
                HostStage::ShuchuiTargetLoop => {
                    let Some(target) = self.target else {
                        return Flow::Done;
                    };
                    if ctx.core.player(target).alive {
                        let equips: Vec<CardId> = ctx.core.player(target).equips.clone();
                        if !equips.is_empty() {
                            for eq in &equips {
                                ctx.take_owned_card(target, *eq);
                                ctx.to_discard(*eq);
                            }
                            ctx.log(LogEvent::CardDiscarded {
                                player: target,
                                cards: equips,
                            });
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
