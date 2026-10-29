use super::super::*;

impl HostMachine {
    pub(super) fn step_swd_damage_targets(&mut self, ctx: &mut Ctx<'_>) -> Flow {
        match self.behavior {
            // 凌锋：TargetChoice 攻击距离内 → Confirm 窗（Confirm=1 伤，
            // 取消=自获 1 护甲）
            HostBehavior::SwdLingfeng => match self.stage {
                HostStage::Act => {
                    // 出牌数门：cards_used_play_phase >= min(3, hp)
                    let need = (3u32).min(ctx.core.player(self.owner).hp as u32);
                    if ctx.core.turn.cards_used_play_phase < need {
                        return Flow::Done;
                    }
                    let options: Vec<ChoiceOption> = ctx
                        .core
                        .alive_players()
                        .filter(|p| {
                            *p != self.owner
                                && ctx.core.computed_distance(ctx.catalog, self.owner, *p)
                                    <= ctx.core.attack_range(ctx.catalog, self.owner)
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
                            prompt: "lingfeng_target",
                            options,
                            min: 1,
                            max: 1,
                            allow_cancel: true,
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
                    self.target = Some(t);
                    self.stage = HostStage::SelectNature;
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::Confirm,
                            prompt: "lingfeng_damage_or_hujia",
                            options: vec![ChoiceOption::Confirm],
                            min: 0,
                            max: 1,
                            allow_cancel: true,
                        },
                    }
                }
                HostStage::SelectNature => {
                    let Some(t) = self.target else {
                        return Flow::Done;
                    };
                    if matches!(ctx.answer().first(), Some(ChoiceOption::Confirm)) {
                        if ctx.core.player(t).alive {
                            ctx.emit(EventData::Damage(crate::machine::damage_data_with(
                                Some(self.owner),
                                t,
                                1,
                                crate::event::DamageNature::Normal,
                                None,
                            )));
                        }
                    } else {
                        ctx.change_hujia(self.owner, 1);
                    }
                    Flow::Done
                }
                _ => Flow::Done,
            },
            // 玄影·端蒙面（受伤反击面）：解除横置 + 对来源虚拟杀
            //（ActivateSkill 面走下方 stage 分支——两入口按触发帧区分：
            // 帧存在且为 Damage=反击面；无帧=主动面）
            HostBehavior::SwdXuanying => {
                let frame_dmg = ctx
                    .res
                    .frame(ctx.current_frame_id())
                    .map(|f| &f.data)
                    .and_then(|data| match data {
                        crate::event::EventData::Damage(d) => Some(d.source),
                        _ => None,
                    });
                if let Some(Some(src)) = frame_dmg {
                    if !ctx.core.player(src).alive {
                        return Flow::Done;
                    }
                    // 已横置（filter 保证）→ commit_linked 翻转为未横置
                    ctx.emit(EventData::Linked {
                        player: self.owner,
                        linked: false,
                    });
                    if !ctx.core.deck.is_empty() {
                        ctx.use_virtual_card(self.owner, CardKind::Sha, vec![src]);
                    }
                    return Flow::Done;
                }
                // 主动面（ActivateSkill）：TargetChoice 任一其他存活 →
                // 横置自身 + 虚拟杀
                match self.stage {
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
                                prompt: "xuanying_target",
                                options,
                                min: 1,
                                max: 1,
                                allow_cancel: true,
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
                            // commit_linked 翻转旗标（不含预置）
                            ctx.emit(EventData::Linked {
                                player: self.owner,
                                linked: true,
                            });
                            if !ctx.core.deck.is_empty() {
                                ctx.use_virtual_card(self.owner, CardKind::Sha, vec![t]);
                            }
                        }
                        Flow::Done
                    }
                    _ => Flow::Done,
                }
            }
            // 聚悉：扣存活数并选一名角色——已受伤则自选受伤/回复，否则受伤
            HostBehavior::SwdJuxi => match self.stage {
                HostStage::Act => {
                    let options: Vec<ChoiceOption> =
                        ctx.core.alive_players().map(ChoiceOption::Player).collect();
                    if options.is_empty() {
                        return Flow::Done;
                    }
                    self.stage = HostStage::TargetChoice;
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectPlayers,
                            prompt: "juxi_target",
                            options,
                            min: 1,
                            max: 1,
                            allow_cancel: true,
                        },
                    }
                }
                HostStage::TargetChoice => {
                    let Some(ChoiceOption::Player(t)) = ctx.answer().first().copied() else {
                        return Flow::Done;
                    };
                    self.target = Some(t);
                    let alive = ctx.core.alive_players().count() as u8;
                    let cur = ctx.core.player(self.owner).juxi_count;
                    ctx.core.player_mut(self.owner).juxi_count = cur.saturating_sub(alive);
                    let damaged = ctx.core.player(t).hp < ctx.core.player(t).max_hp as i32;
                    if damaged {
                        self.stage = HostStage::SelectNature;
                        Flow::WaitChoice {
                            observer: self.owner,
                            spec: ChoiceSpec {
                                kind: ChoiceKind::PickTag,
                                prompt: "juxi_choice",
                                options: vec![ChoiceOption::Tag(0), ChoiceOption::Tag(1)],
                                min: 1,
                                max: 1,
                                allow_cancel: false,
                            },
                        }
                    } else {
                        ctx.emit(EventData::Damage(crate::machine::damage_data_with(
                            Some(self.owner),
                            t,
                            1,
                            crate::event::DamageNature::Normal,
                            None,
                        )));
                        Flow::Done
                    }
                }
                HostStage::SelectNature => {
                    let Some(t) = self.target else {
                        return Flow::Done;
                    };
                    match ctx.answer().first().copied() {
                        Some(ChoiceOption::Tag(1)) => {
                            ctx.recover(t, 1, None);
                        }
                        _ => {
                            ctx.emit(EventData::Damage(crate::machine::damage_data_with(
                                Some(self.owner),
                                t,
                                1,
                                crate::event::DamageNature::Normal,
                                None,
                            )));
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
