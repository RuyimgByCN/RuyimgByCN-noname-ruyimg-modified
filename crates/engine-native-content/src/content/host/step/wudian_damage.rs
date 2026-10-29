use super::super::*;

impl HostMachine {
    pub(super) fn step_wudian_damage(&mut self, ctx: &mut Ctx<'_>) -> Flow {
        match self.behavior {
            HostBehavior::WudianPaotaiFire => {
                // 炮台射击（U05 ow，规则 v123）：End 阶段开始强制触发——
                // 概率 30%×层（RuleRng）→ 随机一名存活敌人 1 点火焰伤害
                //（无敌人则不触发；上游 paotai2 forced 同口径）。
                let lv = ctx.core.player(self.owner).paotai_lv;
                if lv > 0 {
                    let roll = ctx.core.rng.gen_range(100);
                    if roll < 30 * lv as u32 {
                        let cands: Vec<crate::ids::PlayerId> = ctx
                            .core
                            .alive_players()
                            .filter(|p| *p != self.owner)
                            .collect();
                        if !cands.is_empty() {
                            let i = ctx.core.rng.gen_range(cands.len() as u32) as usize;
                            let t = cands[i];
                            ctx.emit(EventData::Damage(crate::machine::damage_data_with(
                                Some(self.owner),
                                t,
                                1,
                                crate::event::DamageNature::Fire,
                                None,
                            )));
                        }
                    }
                }
                Flow::Done
            }
            HostBehavior::WudianFeidan => match self.stage {
                // 飞弹（U05 ow 法老之鹰，规则 v130）：Act 建弹射目标与
                // 弃牌选项→SelectCard 读弃牌应答（取消=不弃不弹）→按
                // 座位序逐目标各受 1 点无牌伤害（source=持有者；上游
                // damage() 无参同口径）。
                HostStage::Act => {
                    // 受伤者从触发事件带入 target 字段（machine() 分流）
                    let Some(hurt) = self.target else {
                        return Flow::Done;
                    };
                    if !ctx.core.player(hurt).alive {
                        return Flow::Done;
                    }
                    let options: Vec<ChoiceOption> = ctx
                        .core
                        .alive_players()
                        .filter(|p| {
                            *p != hurt && ctx.core.computed_distance(ctx.catalog, hurt, *p) <= 1
                        })
                        .map(ChoiceOption::Player)
                        .collect();
                    if options.is_empty() {
                        return Flow::Done;
                    }
                    self.targets = options
                        .iter()
                        .filter_map(|o| match o {
                            ChoiceOption::Player(p) => Some(*p),
                            _ => None,
                        })
                        .collect();
                    let he: Vec<ChoiceOption> = {
                        let p = ctx.core.player(self.owner);
                        p.hand
                            .iter()
                            .copied()
                            .chain(p.equips.iter().copied())
                            .map(ChoiceOption::Card)
                            .collect()
                    };
                    if he.is_empty() {
                        return Flow::Done;
                    }
                    self.stage = HostStage::SelectCard;
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectCards,
                            prompt: WUDIAN_FEIDAN_DISCARD,
                            options: he,
                            min: 1,
                            max: 1,
                            allow_cancel: true,
                        },
                    }
                }
                HostStage::SelectCard => {
                    let disc = ctx.answer().first().and_then(|o| match o {
                        ChoiceOption::Card(c) => Some(*c),
                        _ => None,
                    });
                    let Some(c) = disc else {
                        return Flow::Done; // 放弃：不弃牌不弹射
                    };
                    ctx.take_owned_card(self.owner, c);
                    ctx.to_discard(c);
                    ctx.log(LogEvent::CardDiscarded {
                        player: self.owner,
                        cards: vec![c],
                    });
                    // 按座位序逐目标伤害（上游 sort seat + shift damage）
                    let hurt = self.target.unwrap_or(self.owner);
                    let mut sorted = std::mem::take(&mut self.targets);
                    sorted.sort_by_key(|p| ctx.core.seat_steps_from(hurt, *p));
                    for t in sorted {
                        if ctx.core.player(t).alive {
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
