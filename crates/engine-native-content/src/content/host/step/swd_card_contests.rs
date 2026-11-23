use super::super::*;
use super::u08_batch_d::{flag, set_flag};

impl HostMachine {
    pub(super) fn step_swd_card_contests(&mut self, ctx: &mut Ctx<'_>) -> Flow {
        match self.behavior {
            // 对诗：Act 弃一张手牌 → SelectCard 指定目标 → TargetChoice
            // 目标同花色弃牌窗 → CardsAsk 消费（空选=拒绝→随机获得其一
            // 张）。usable:1 收敛上游 duishi2/duishi3 临时技面（登记差异）。
            HostBehavior::SwdDuishi => match self.stage {
                HostStage::Act => {
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
                    self.stage = HostStage::SelectCard;
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectCards,
                            prompt: "duishi_discard",
                            options,
                            min: 1,
                            max: 1,
                            allow_cancel: true,
                        },
                    }
                }
                HostStage::SelectCard => {
                    let Some(picked) = ctx.answer().iter().find_map(|o| match o {
                        ChoiceOption::Card(c) => Some(*c),
                        _ => None,
                    }) else {
                        return Flow::Done;
                    };
                    if !ctx.core.player(self.owner).hand.contains(&picked) {
                        return Flow::Done;
                    }
                    self.card = Some(picked);
                    self.cards = vec![picked];
                    let options: Vec<ChoiceOption> = ctx
                        .core
                        .alive_players()
                        .filter(|p| {
                            *p != self.owner
                                && !ctx.core.player(*p).hand.is_empty()
                                && flag(ctx.core, *p, "swd_duishi3") == 0
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
                            prompt: "duishi_target",
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
                    let Some(picked) = self.card else {
                        return Flow::Done;
                    };
                    if !ctx.core.player(t).alive
                        || ctx.core.player(t).hand.is_empty()
                        || !ctx.core.player(self.owner).hand.contains(&picked)
                    {
                        return Flow::Done;
                    }
                    ctx.discard_from_hand(self.owner, picked);
                    let suit = ctx.catalog.effective_card_suit(ctx.core, self.owner, picked);
                    let options: Vec<ChoiceOption> = ctx
                        .core
                        .player(t)
                        .hand
                        .iter()
                        .filter(|c| ctx.catalog.effective_card_suit(ctx.core, t, **c) == suit)
                        .copied()
                        .map(ChoiceOption::Card)
                        .collect();
                    self.target = Some(t);
                    if options.is_empty() {
                        return self.duishi_prompt_gain(ctx, t);
                    }
                    self.stage = HostStage::CardsAsk;
                    Flow::WaitChoice {
                        observer: t,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectCards,
                            prompt: "duishi_discard_same_suit",
                            options,
                            min: 1,
                            max: 1,
                            allow_cancel: true,
                        },
                    }
                }
                HostStage::CardsAsk => {
                    let pick = ctx.answer().iter().find_map(|o| match o {
                        ChoiceOption::Card(c) => Some(*c),
                        _ => None,
                    });
                    let Some(t) = self.target else {
                        return Flow::Done;
                    };
                    if let Some(c) = pick {
                        if ctx.core.player(t).hand.contains(&c) {
                            ctx.discard_from_hand(t, c);
                            set_flag(ctx.core, t, "swd_duishi3", 1);
                            return Flow::Done;
                        }
                    }
                    self.duishi_prompt_gain(ctx, t)
                }
                HostStage::CompareTargetPick => {
                    let Some(t) = self.target else {
                        return Flow::Done;
                    };
                    let Some(ChoiceOption::Card(c)) = ctx.answer().first().copied() else {
                        return Flow::Done;
                    };
                    let in_hand = ctx.core.player(t).hand.contains(&c);
                    let in_equip = ctx.core.player(t).equips.contains(&c);
                    if !in_hand && !in_equip {
                        return Flow::Done;
                    }
                    let orig = if in_hand {
                        ctx.take_hand_card(t, c);
                        crate::event::LostOrigin::Hand
                    } else {
                        ctx.core.player_mut(t).equips.retain(|x| *x != c);
                        ctx.core.declared_slots.remove(&c);
                        ctx.core.player_mut(t).card_mods.remove(&c);
                        if ctx.catalog.card_kind(c).is_equip() {
                            crate::kernel::revoke_equip_skills(ctx.core, ctx.catalog, t, c);
                        }
                        crate::event::LostOrigin::Equip
                    };
                    ctx.note_cards_lost(t, &[c]);
                    if ctx.core.player(self.owner).alive {
                        ctx.core.player_mut(self.owner).hand.push(c);
                        ctx.log(LogEvent::CardsTaken {
                            from: t,
                            to: self.owner,
                            cards: vec![c],
                            public: !in_hand,
                        });
                        set_flag(ctx.core, self.owner, "swd_duishi2", 1);
                        ctx.emit(EventData::CardsMoved {
                            cards: vec![c],
                            from: t,
                            to: self.owner,
                        });
                    } else {
                        ctx.to_discard(c);
                    }
                    ctx.emit(EventData::CardsLost {
                        player: t,
                        cards: vec![c],
                        origins: vec![orig],
                    });
                    Flow::Done
                }
                _ => Flow::Done,
            },
            // 天书：弃一张锦囊手牌 → 选角色 → 选其一个未持有技能 → 永久获取
            HostBehavior::SwdTianshu => match self.stage {
                HostStage::Act => {
                    let options: Vec<ChoiceOption> = ctx
                        .core
                        .player(self.owner)
                        .hand
                        .iter()
                        .copied()
                        .filter(|c| ctx.catalog.card_kind(*c).is_trick_kind())
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
                            prompt: "swdtianshu_card",
                            options,
                            min: 1,
                            max: 1,
                            allow_cancel: false,
                        },
                    }
                }
                HostStage::SelectCard => {
                    let Some(ChoiceOption::Card(card)) = ctx.answer().first().copied() else {
                        return Flow::Done;
                    };
                    if !ctx.core.player(self.owner).hand.contains(&card) {
                        return Flow::Done;
                    }
                    self.card = Some(card);
                    let options: Vec<ChoiceOption> = ctx
                        .core
                        .alive_players()
                        .filter(|p| {
                            *p != self.owner && !swdtianshu_skills(ctx, *p, self.owner).is_empty()
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
                            prompt: "swdtianshu_target",
                            options,
                            min: 1,
                            max: 1,
                            allow_cancel: false,
                        },
                    }
                }
                HostStage::TargetChoice => {
                    let Some(ChoiceOption::Player(target)) = ctx.answer().first().copied() else {
                        return Flow::Done;
                    };
                    let names = swdtianshu_skills(ctx, target, self.owner);
                    if names.is_empty() {
                        return Flow::Done;
                    }
                    self.target = Some(target);
                    // Tag=候选下标（answer 时按同序重枚举映射——技能定义 id
                    // 可能超出 u8）
                    let options: Vec<ChoiceOption> = (0..names.len())
                        .map(|i| ChoiceOption::Tag(i as u8))
                        .collect();
                    if options.is_empty() {
                        return Flow::Done;
                    }
                    self.stage = HostStage::ShuchuiTargetLoop;
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::PickTag,
                            prompt: "swdtianshu_skill",
                            options,
                            min: 1,
                            max: 1,
                            allow_cancel: false,
                        },
                    }
                }
                HostStage::ShuchuiTargetLoop => {
                    let Some(ChoiceOption::Tag(idx)) = ctx.answer().first().copied() else {
                        return Flow::Done;
                    };
                    let Some(card) = self.card else {
                        return Flow::Done;
                    };
                    let Some(target) = self.target else {
                        return Flow::Done;
                    };
                    // 重枚举映射（同序）：下标 → 技能定义
                    let names = swdtianshu_skills(ctx, target, self.owner);
                    let Some(name) = names.get(idx as usize).copied() else {
                        return Flow::Done;
                    };
                    let Some(def) = ctx.catalog.skill_by_name(name) else {
                        return Flow::Done;
                    };
                    let def_id = u16::from(def);
                    ctx.log(LogEvent::SkillTriggered {
                        skill: "天书",
                        owner: self.owner, presentation: None });
                    // 弃置锦囊手牌（上游 phaseUse 用牌消耗）
                    if ctx.core.player(self.owner).hand.contains(&card) {
                        ctx.discard_from_hand(self.owner, card);
                    }
                    // 永久获取（附加技实例；先清除此前天书附加技）
                    if let Some(old_raw) = ctx.core.player_mut(self.owner).swdtianshu_granted.take() {
                        let old_def = crate::ids::SkillDefId::from(old_raw);
                        if old_def != def {
                            ctx.core.skills.remove(&(self.owner, old_def));
                        }
                    }
                    if !ctx.core.skills.contains_key(&(self.owner, def)) {
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
                    {
                        let p = ctx.core.player_mut(self.owner);
                        p.swdtianshu_from = Some(target);
                        p.swdtianshu_granted = Some(u16::from(def_id));
                    }
                    ctx.log(LogEvent::SkillToggled {
                        player: self.owner,
                        skill: ctx.catalog.skill_name(def),
                        enabled: true,
                    });
                    Flow::Done
                }
                _ => Flow::Done,
            },
            _ => unreachable!("behavior routed to the wrong step module"),
        }
    }
}
