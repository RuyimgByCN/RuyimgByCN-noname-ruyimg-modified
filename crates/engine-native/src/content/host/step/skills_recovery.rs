use super::super::*;

impl HostMachine {
    pub(super) fn step_skills_recovery(&mut self, ctx: &mut Ctx<'_>) -> Flow {
        match self.behavior {
            // 狂骨：选「回复 1」或「摸 1」
            HostBehavior::FzXinKuanggu => match self.stage {
                HostStage::Act => {
                    self.stage = HostStage::SelectNature;
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::PickTag,
                            prompt: "fz_xinkuanggu_choice",
                            // 0=回复 1 点体力；1=摸 1 张牌
                            options: vec![ChoiceOption::Tag(0), ChoiceOption::Tag(1)],
                            min: 1,
                            max: 1,
                            allow_cancel: false,
                        },
                    }
                }
                HostStage::SelectNature => {
                    let Some(ChoiceOption::Tag(choice)) = ctx.answer().first().copied() else {
                        return Flow::Done;
                    };
                    if choice == 0 {
                        ctx.recover(self.owner, 1, None);
                    } else {
                        ctx.draw_cards(self.owner, 1);
                    }
                    ctx.log(LogEvent::SkillTriggered {
                        skill: "狂骨",
                        owner: self.owner,
                    });
                    Flow::Done
                }
                _ => Flow::Done,
            },
            // 血量翻倍：体力上限 + 当前体力并回复等量（一次性）
            HostBehavior::XiguaXueliangFanbei => {
                let hp = ctx.core.player(self.owner).hp.max(0);
                if hp > 0 {
                    ctx.core.player_mut(self.owner).max_hp += hp as u32;
                    ctx.recover(self.owner, hp as u32, None);
                }
                if let Some(def) = ctx.catalog.skill_by_name(L20_XUELIANG_KEY) {
                    ctx.core
                        .vm_aux
                        .marks
                        .insert((self.owner, u32::from(def)), 1);
                }
                ctx.log(LogEvent::SkillTriggered {
                    skill: "血量翻倍",
                    owner: self.owner,
                });
                Flow::Done
            }
            // 誓师：体力上限 +1 并回复 1
            HostBehavior::Shishi => {
                ctx.core.player_mut(self.owner).max_hp += 1;
                ctx.recover(self.owner, 1, None);
                ctx.log(LogEvent::SkillTriggered {
                    skill: "誓师",
                    owner: self.owner,
                });
                Flow::Done
            }
            // 魔化：移除旧三技 → 获得撒旦四技 → 上限 4 → 回复 2 → 摸 2
            HostBehavior::Mohua2 => {
                for name in ["miles_xueyi", "aojian", "mohua2"] {
                    if let Some(def) = ctx.catalog.skill_by_name(name) {
                        if ctx.core.skills.remove(&(self.owner, def)).is_some() {
                            ctx.log(LogEvent::SkillToggled {
                                player: self.owner,
                                skill: ctx.catalog.skill_name(def),
                                enabled: false,
                            });
                        }
                    }
                }
                for name in ["moyan", "miedao", "jifeng", "swd_xiuluo"] {
                    let Some(def) = ctx.catalog.skill_by_name(name) else {
                        continue;
                    };
                    if !ctx.core.skills.contains_key(&(self.owner, def)) {
                        ctx.core.skills.insert(
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
                    ctx.log(LogEvent::SkillToggled {
                        player: self.owner,
                        skill: ctx.catalog.skill_name(def),
                        enabled: true,
                    });
                }
                // 撒旦形态体力上限 4（上游 lib.character.swd_satan 同口径）
                {
                    let p = ctx.core.player_mut(self.owner);
                    p.max_hp = 4;
                    if p.hp > 4 {
                        p.hp = 4;
                    }
                }
                ctx.log(LogEvent::SkillTriggered {
                    skill: "魔化",
                    owner: self.owner,
                });
                ctx.recover(self.owner, 2, None);
                ctx.draw_cards(self.owner, 2);
                Flow::Done
            }
            // 脉轮：选 1..8 效果 → 立即结算（1/2）或置模式位（3..8）
            HostBehavior::Mailun => match self.stage {
                HostStage::Act => {
                    let max_hp = ctx.core.player(self.owner).max_hp;
                    // 上游：maxHp==6 时不提供效果 1
                    let options: Vec<ChoiceOption> = (1..=8u8)
                        .filter(|i| !(*i == 1 && max_hp == 6))
                        .map(ChoiceOption::Tag)
                        .collect();
                    self.stage = HostStage::SelectNature;
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::PickTag,
                            prompt: "mailun_mode",
                            options,
                            min: 1,
                            max: 1,
                            allow_cancel: true,
                        },
                    }
                }
                HostStage::SelectNature => {
                    let Some(ChoiceOption::Tag(mode)) = ctx.answer().first().copied() else {
                        return Flow::Done;
                    };
                    ctx.log(LogEvent::SkillTriggered {
                        skill: "脉轮",
                        owner: self.owner,
                    });
                    // 清除旧效果（上游 step0 移除 mailun31..8）
                    {
                        let p = ctx.core.player_mut(self.owner);
                        p.mailun_mode = 0;
                        p.mailun_out_used = false;
                        p.mailun_in_used = false;
                    }
                    match mode {
                        1 => {
                            // 减少一点体力并增加一点体力上限
                            let max = ctx.core.player(self.owner).max_hp;
                            ctx.emit(EventData::LoseHp {
                                player: self.owner,
                                amount: 1,
                            });
                            ctx.core.player_mut(self.owner).max_hp = max + 1;
                        }
                        2 => {
                            // 增加一点体力并减少一点体力上限
                            ctx.recover(self.owner, 1, None);
                            ctx.lose_max_hp(self.owner, 1);
                        }
                        m => {
                            ctx.core.player_mut(self.owner).mailun_mode = m;
                        }
                    }
                    Flow::Done
                }
                _ => Flow::Done,
            },
            // 护天：选任意张牌 → 选体力上限足够的目标 → 寄存 + 补至张数
            HostBehavior::Hutian => match self.stage {
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
                        return Flow::Done;
                    }
                    let max = options.len();
                    self.stage = HostStage::SelectCard;
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectCards,
                            prompt: "hutian_cards",
                            options,
                            min: 1,
                            max,
                            allow_cancel: true,
                        },
                    }
                }
                HostStage::SelectCard => {
                    let picked: Vec<CardId> = ctx
                        .answer()
                        .iter()
                        .filter_map(|o| match o {
                            ChoiceOption::Card(c) => Some(*c),
                            _ => None,
                        })
                        .collect();
                    if picked.is_empty() {
                        return Flow::Done;
                    }
                    self.cards = picked.iter().map(|c| u32::from(*c)).collect();
                    let need = picked.len() as u16;
                    let options: Vec<ChoiceOption> = ctx
                        .core
                        .alive_players()
                        .filter(|p| *p != self.owner && ctx.core.player(*p).max_hp as u16 >= need)
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
                            prompt: "hutian_target",
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
                    let cards: Vec<CardId> = self.cards.iter().map(|c| *c as CardId).collect();
                    let count = cards.len() as i32;
                    // 牌离场入寄存区（手牌移除 / 装备撤技后移除；不入弃堆）
                    for c in cards.iter().copied() {
                        if ctx.core.player(self.owner).hand.contains(&c) {
                            ctx.take_hand_card(self.owner, c);
                        } else if ctx.core.player(self.owner).equips.contains(&c) {
                            ctx.core.player_mut(self.owner).equips.retain(|x| *x != c);
                            crate::kernel::revoke_equip_skills(
                                ctx.core,
                                ctx.catalog,
                                self.owner,
                                c,
                            );
                        } else {
                            return Flow::Done;
                        }
                    }
                    {
                        let p = ctx.core.player_mut(self.owner);
                        p.hutian_deposit = Some(target);
                        p.hutian_just_given = true;
                    }
                    {
                        let t = ctx.core.player_mut(target);
                        t.hutian_owner = Some(self.owner);
                        t.hutian_hold.extend(cards.iter().copied());
                    }
                    ctx.log(LogEvent::SkillTriggered {
                        skill: "护天",
                        owner: self.owner,
                    });
                    // 目标体力低于寄存数 → 回复至该数（上游 step2）
                    let hp = ctx.core.player(target).hp;
                    if hp < count {
                        ctx.recover(target, (count - hp) as u32, None);
                    }
                    Flow::Done
                }
                _ => Flow::Done,
            },
            HostBehavior::PhaseEndRecoverOrDraw => {
                let p = ctx.core.player(self.owner);
                if p.hp < p.max_hp as i32 {
                    ctx.recover(self.owner, 1, None);
                } else {
                    ctx.draw_cards(self.owner, 1);
                }
                Flow::Done
            }
            HostBehavior::WangmeiZhikeCard => {
                // 条件增益（上游 content）：最小体力且已受伤 → 回复 1；
                // 最小手牌数 → 摸（已回复 ? 1 : 2）张
                let Some(t) = self.target else {
                    return Flow::Done;
                };
                if !ctx.core.player(t).alive {
                    return Flow::Done;
                }
                let mut min_hp = i32::MAX;
                let mut min_hand = usize::MAX;
                for p in ctx.core.alive_players() {
                    let pl = ctx.core.player(p);
                    min_hp = min_hp.min(pl.hp);
                    min_hand = min_hand.min(pl.hand.len());
                }
                let tp = ctx.core.player(t);
                let damaged = tp.hp < tp.max_hp as i32;
                let is_min_hp = tp.hp == min_hp;
                let is_min_hand = tp.hand.len() == min_hand;
                let mut rec = false;
                if is_min_hp && damaged {
                    ctx.recover(t, 1, self.card);
                    rec = true;
                }
                if is_min_hand {
                    ctx.draw_cards(t, if rec { 1 } else { 2 });
                }
                if let Some(c) = self.card {
                    ctx.to_discard(c);
                    ctx.core.table.retain(|x| *x != c);
                }
                Flow::Done
            }
            _ => unreachable!("behavior routed to the wrong step module"),
        }
    }
}
