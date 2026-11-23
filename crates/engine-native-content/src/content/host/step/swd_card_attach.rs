//! 轩辕剑（swd）宿主技：把牌置于目标武将牌旁或判定区（枕流、麻醉、玄咒、命符）。

use super::super::*;
use super::u08_batch_d::set_flag;

impl HostMachine {
    pub(super) fn step_swd_card_attach(&mut self, ctx: &mut Ctx<'_>) -> Flow {
        match self.behavior {
            // 枕流·临月面：SelectCard 红色手牌 → TargetChoice 其他未挂附
            // 存活 → off_court 挂附（上游 placeCard 武将牌上同口径；
            // 触发面 SwdZhenjiu2 在其准备阶段取回）
            HostBehavior::SwdZhenjiu => match self.stage {
                HostStage::Act => {
                    let options: Vec<ChoiceOption> = ctx
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
                            prompt: "zhenjiu_pick",
                            options,
                            min: 1,
                            max: 1,
                            allow_cancel: true,
                        },
                    }
                }
                HostStage::SelectCard => {
                    let Some(c) = ctx.answer().iter().find_map(|o| match o {
                        ChoiceOption::Card(c) => Some(*c),
                        _ => None,
                    }) else {
                        return Flow::Done;
                    };
                    if !ctx.core.player(self.owner).hand.contains(&c)
                        || !matches!(
                            ctx.catalog.cards[c as usize].suit,
                            crate::catalog::Suit::Heart | crate::catalog::Suit::Diamond
                        )
                    {
                        return Flow::Done;
                    }
                    self.card = Some(c);
                    let options: Vec<ChoiceOption> = ctx
                        .core
                        .alive_players()
                        .filter(|p| *p != self.owner && ctx.core.player(*p).off_court.is_empty())
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
                            prompt: "zhenjiu_target",
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
                    let Some(c) = self.card else {
                        return Flow::Done;
                    };
                    if ctx.core.player(t).alive
                        && ctx.core.player(t).off_court.is_empty()
                        && ctx.core.player(self.owner).hand.contains(&c)
                    {
                        ctx.take_hand_card(self.owner, c);
                        ctx.core.player_mut(t).off_court.push(c);
                        // 授予触发面实例（其准备阶段取回+回复后自清）
                        if let Some(def) = ctx.catalog.skill_by_name(SWD_ZHENJIU2_KEY) {
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
                    Flow::Done
                }
                _ => Flow::Done,
            },
            // 麻醉·临月面：SelectCard 黑色手牌 → TargetChoice → off_court
            // 挂附（触发面 SwdLmazui2 在持有者伤害前置返还+降 1）
            HostBehavior::SwdLmazui => match self.stage {
                HostStage::Act => {
                    let options: Vec<ChoiceOption> = ctx
                        .core
                        .player(self.owner)
                        .hand
                        .iter()
                        .copied()
                        .filter(|c| {
                            matches!(
                                ctx.catalog.effective_card_suit(ctx.core, self.owner, *c),
                                crate::catalog::Suit::Spade | crate::catalog::Suit::Club
                            )
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
                            prompt: "lmazui_pick",
                            options,
                            min: 1,
                            max: 1,
                            allow_cancel: true,
                        },
                    }
                }
                HostStage::SelectCard => {
                    let Some(c) = ctx.answer().iter().find_map(|o| match o {
                        ChoiceOption::Card(c) => Some(*c),
                        _ => None,
                    }) else {
                        super::u05_7c_skills::refund_once_used(ctx, self.owner, &self.def_key);
                        return Flow::Done;
                    };
                    if !ctx.core.player(self.owner).hand.contains(&c)
                        || !matches!(
                            ctx.catalog.effective_card_suit(ctx.core, self.owner, c),
                            crate::catalog::Suit::Spade | crate::catalog::Suit::Club
                        )
                    {
                        super::u05_7c_skills::refund_once_used(ctx, self.owner, &self.def_key);
                        return Flow::Done;
                    }
                    self.card = Some(c);
                    let lm2_def = ctx.catalog.skill_by_name(SWD_LMAZUI2_KEY);
                    let options: Vec<ChoiceOption> = ctx
                        .core
                        .alive_players()
                        .filter(|p| !lm2_def.is_some_and(|d| ctx.core.has_skill(*p, d)))
                        .map(ChoiceOption::Player)
                        .collect();
                    if options.is_empty() {
                        super::u05_7c_skills::refund_once_used(ctx, self.owner, &self.def_key);
                        return Flow::Done;
                    }
                    self.stage = HostStage::TargetChoice;
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectPlayers,
                            prompt: "lmazui_target",
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
                        super::u05_7c_skills::refund_once_used(ctx, self.owner, &self.def_key);
                        return Flow::Done;
                    };
                    let Some(c) = self.card else {
                        super::u05_7c_skills::refund_once_used(ctx, self.owner, &self.def_key);
                        return Flow::Done;
                    };
                    let lm2_def = ctx.catalog.skill_by_name(SWD_LMAZUI2_KEY);
                    if ctx.core.player(t).alive
                        && !lm2_def.is_some_and(|d| ctx.core.has_skill(t, d))
                        && ctx.core.player(self.owner).hand.contains(&c)
                    {
                        ctx.take_hand_card(self.owner, c);
                        ctx.core.player_mut(t).off_court.push(c);
                        set_flag(ctx.core, t, "swd_lmazui2_card", u32::from(c) + 1);
                        // 授予触发面实例（其伤害前置降 1 后挂 lmazui3 归还）
                        if let Some(def) = lm2_def {
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
                    Flow::Done
                }
                _ => Flow::Done,
            },
            // 玄咒：SelectCard 置出一张普通锦囊（discard:false=置出而非
            // 弃——原生近似弃置登记）→ TargetChoice 任一角色（含自己，
            // 判定区无同名牌）→ 确定性 kind 规则置入判定区
            HostBehavior::SwdXuanzhou => match self.stage {
                HostStage::Act => {
                    let tricks: Vec<ChoiceOption> = ctx
                        .core
                        .player(self.owner)
                        .hand
                        .iter()
                        .copied()
                        .filter(|c| {
                            let k = ctx.catalog.card_kind(*c);
                            !k.is_basic_kind() && k.equip_slot().is_none() && k != CardKind::Jiguan
                        })
                        .map(ChoiceOption::Card)
                        .collect();
                    if tricks.is_empty() {
                        return Flow::Done;
                    }
                    self.stage = HostStage::SelectCard;
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectCards,
                            prompt: "xuanzhou_trick",
                            options: tricks,
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
                    let options: Vec<ChoiceOption> = ctx
                        .core
                        .alive_players()
                        .filter(|p| {
                            // 判定区无同名延时（乐/兵互不冲突）
                            ctx.core.player(*p).judge.iter().all(|j| {
                                let jk = ctx.catalog.card_kind(*j);
                                jk != CardKind::Le && jk != CardKind::Bing
                            })
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
                            prompt: "xuanzhou_target",
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
                    let Some(c) = self.card else {
                        return Flow::Done;
                    };
                    if !ctx.core.player(t).alive {
                        return Flow::Done;
                    }
                    // 确定性 kind：目标手牌≤1→兵；余→乐（上游 AI 序近似，
                    // 玩家自选面登记差异）
                    let kind = if ctx.core.player(t).hand.len() <= 1 {
                        CardKind::Bing
                    } else {
                        CardKind::Le
                    };
                    ctx.take_hand_card(self.owner, c);
                    ctx.to_discard(c);
                    // 延时牌置入判定区：直接造该 kind 的判定实体
                    let judge_card = ctx
                        .core
                        .deck
                        .iter()
                        .copied()
                        .find(|x| ctx.catalog.card_kind(*x) == kind)
                        .or_else(|| {
                            let found = ctx
                                .core
                                .discard
                                .iter()
                                .copied()
                                .find(|x| ctx.catalog.card_kind(*x) == kind);
                            found
                        });
                    let Some(jc) = judge_card else {
                        return Flow::Done;
                    };
                    if ctx.core.deck.contains(&jc) {
                        ctx.core.deck.retain(|x| *x != jc);
                    } else {
                        ctx.core.discard.retain(|x| *x != jc);
                    }
                    ctx.core.player_mut(t).judge.push(jc);
                    ctx.emit(EventData::CardsMoved {
                        cards: vec![jc],
                        from: self.owner,
                        to: t,
                    });
                    self.card = None;
                    Flow::Done
                }
                _ => Flow::Done,
            },
            // 命符·姜子牙面（swd/skill.js:3844）：SelectCard 暂存梅花 he 牌 → TargetChoice 任一
            // 判定区无鬼幽结的角色 → 不弃牌直接将该牌声明为鬼幽结置入其判定区；取消退回 once_used
            HostBehavior::SwdMingfu => match self.stage {
                HostStage::Act => {
                    let options: Vec<ChoiceOption> = ctx
                        .core
                        .player(self.owner)
                        .hand
                        .iter()
                        .chain(ctx.core.player(self.owner).equips.iter())
                        .copied()
                        .filter(|c| {
                            matches!(
                                ctx.catalog.effective_card_suit(ctx.core, self.owner, *c),
                                crate::catalog::Suit::Club
                            )
                        })
                        .map(ChoiceOption::Card)
                        .collect();
                    let has_target = ctx.core.alive_players().any(|p| {
                        p != self.owner
                            && !ctx.catalog.blocks_delayed_trick(ctx.core, p)
                            && !ctx.core.player(p).judge.iter().any(|jc| {
                                ctx.core
                                    .declared_kinds
                                    .get(jc)
                                    .copied()
                                    .unwrap_or_else(|| ctx.catalog.card_kind(*jc))
                                    == CardKind::Guiyoujie
                            })
                    });
                    if options.is_empty() || !has_target {
                        super::u05_7c_skills::refund_once_used(ctx, self.owner, &self.def_key);
                        return Flow::Done;
                    }
                    self.stage = HostStage::SelectCard;
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectCards,
                            prompt: "mingfu_pick",
                            options,
                            min: 1,
                            max: 1,
                            allow_cancel: true,
                        },
                    }
                }
                HostStage::SelectCard => {
                    let Some(c) = ctx.answer().iter().find_map(|o| match o {
                        ChoiceOption::Card(c) => Some(*c),
                        _ => None,
                    }) else {
                        super::u05_7c_skills::refund_once_used(ctx, self.owner, &self.def_key);
                        return Flow::Done;
                    };
                    let owned = ctx.core.player(self.owner).hand.contains(&c)
                        || ctx.core.player(self.owner).equips.contains(&c);
                    if !owned
                        || !matches!(
                            ctx.catalog.effective_card_suit(ctx.core, self.owner, c),
                            crate::catalog::Suit::Club
                        )
                    {
                        super::u05_7c_skills::refund_once_used(ctx, self.owner, &self.def_key);
                        return Flow::Done;
                    }
                    self.card = Some(c);
                    let options: Vec<ChoiceOption> = ctx
                        .core
                        .alive_players()
                        .filter(|p| {
                            *p != self.owner
                                && !ctx.catalog.blocks_delayed_trick(ctx.core, *p)
                                && !ctx.core.player(*p).judge.iter().any(|jc| {
                                    ctx.core
                                        .declared_kinds
                                        .get(jc)
                                        .copied()
                                        .unwrap_or_else(|| ctx.catalog.card_kind(*jc))
                                        == CardKind::Guiyoujie
                                })
                        })
                        .map(ChoiceOption::Player)
                        .collect();
                    if options.is_empty() {
                        super::u05_7c_skills::refund_once_used(ctx, self.owner, &self.def_key);
                        return Flow::Done;
                    }
                    self.stage = HostStage::TargetChoice;
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectPlayers,
                            prompt: "mingfu_target",
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
                        super::u05_7c_skills::refund_once_used(ctx, self.owner, &self.def_key);
                        return Flow::Done;
                    };
                    let Some(c) = self.card.take() else {
                        super::u05_7c_skills::refund_once_used(ctx, self.owner, &self.def_key);
                        return Flow::Done;
                    };
                    if !ctx.core.player(t).alive
                        || t == self.owner
                        || ctx.catalog.blocks_delayed_trick(ctx.core, t)
                    {
                        super::u05_7c_skills::refund_once_used(ctx, self.owner, &self.def_key);
                        return Flow::Done;
                    }
                    if ctx.core.player(self.owner).hand.contains(&c) {
                        ctx.take_hand_card(self.owner, c);
                    } else if ctx.core.player(self.owner).equips.contains(&c) {
                        ctx.core.player_mut(self.owner).equips.retain(|x| *x != c);
                        if ctx.catalog.card_kind(c).is_equip() {
                            crate::kernel::revoke_equip_skills(ctx.core, ctx.catalog, self.owner, c);
                        }
                        ctx.emit(EventData::CardsLost {
                            player: self.owner,
                            cards: vec![c],
                            origins: vec![crate::event::LostOrigin::Equip],
                        });
                    } else {
                        super::u05_7c_skills::refund_once_used(ctx, self.owner, &self.def_key);
                        return Flow::Done;
                    }
                    ctx.core.declared_kinds.insert(c, CardKind::Guiyoujie);
                    ctx.core.player_mut(t).judge.push(c);
                    ctx.emit(EventData::CardsMoved {
                        cards: vec![c],
                        from: self.owner,
                        to: t,
                    });
                    Flow::Done
                }
                _ => Flow::Done,
            },
            _ => unreachable!("behavior routed to the wrong step module"),
        }
    }
}
