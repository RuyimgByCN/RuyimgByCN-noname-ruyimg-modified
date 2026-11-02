use super::super::*;

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
                    // 花色由所选实体查目录——同花色弃牌窗按此过滤
                    self.cards = vec![picked];
                    ctx.discard_from_hand(self.owner, picked);
                    let options: Vec<ChoiceOption> = ctx
                        .core
                        .alive_players()
                        .filter(|p| *p != self.owner && !ctx.core.player(*p).hand.is_empty())
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
                    if !ctx.core.player(t).alive || ctx.core.player(t).hand.is_empty() {
                        return Flow::Done;
                    }
                    let suit = self
                        .cards
                        .first()
                        .map(|c| ctx.catalog.cards[*c as usize].suit);
                    let options: Vec<ChoiceOption> = ctx
                        .core
                        .player(t)
                        .hand
                        .iter()
                        .filter(|c| {
                            suit.map(|s| ctx.catalog.cards[**c as usize].suit == s)
                                .unwrap_or(false)
                        })
                        .copied()
                        .map(ChoiceOption::Card)
                        .collect();
                    if options.is_empty() {
                        // 无同花色手牌：直接走获得面
                        return self.duishi_gain_random(ctx, t);
                    }
                    self.target = Some(t);
                    self.stage = HostStage::CardsAsk;
                    Flow::WaitChoice {
                        observer: t,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectCards,
                            prompt: "duishi_discard_same_suit",
                            options,
                            min: 0,
                            max: 1,
                            allow_cancel: false,
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
                    if let Some(c) = pick.filter(|c| ctx.core.player(t).hand.contains(c)) {
                        ctx.discard_from_hand(t, c);
                        Flow::Done
                    } else {
                        // 空应答=拒绝 → 随机获得
                        self.duishi_gain_random(ctx, t)
                    }
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
                        .filter(|c| {
                            let kind = ctx.catalog.card_kind(*c);
                            !kind.is_basic_kind() && !kind.is_equip()
                        })
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
                        owner: self.owner,
                    });
                    // 弃置锦囊手牌（上游 phaseUse 用牌消耗）
                    if ctx.core.player(self.owner).hand.contains(&card) {
                        ctx.discard_from_hand(self.owner, card);
                    }
                    // 永久获取（附加技实例）
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
