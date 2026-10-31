//! 轩辕剑（swd）宿主技：濒死、受伤或被使用时的弃牌应对窗口（苏生、遁行、玉鳞、凝霜、工神）。

use super::super::*;

impl HostMachine {
    pub(super) fn step_swd_response_cards(&mut self, ctx: &mut Ctx<'_>) -> Flow {
        match self.behavior {
            // 苏生：死亡前置窗口——SelectCard 弃一张手牌（可取消=放弃，
            // 死亡照常）→ 目标体力回至 1 + death_cancelled 写回（空洞印
            // 取消基建；DeathMachine 清 dying 旗）
            HostBehavior::SwdSusheng => match self.stage {
                HostStage::Act => {
                    let Some(p) = ctx
                        .res
                        .frame(ctx.current_frame_id())
                        .map(|f| &f.data)
                        .and_then(|data| match data {
                            crate::event::EventData::Death { player, .. } => Some(*player),
                            _ => None,
                        })
                    else {
                        return Flow::Done;
                    };
                    self.target = Some(p);
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
                            prompt: "susheng_discard",
                            options,
                            min: 0,
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
                    ctx.take_hand_card(self.owner, picked);
                    ctx.to_discard(picked);
                    ctx.log(LogEvent::CardDiscarded {
                        player: self.owner,
                        cards: vec![picked],
                    });
                    let Some(t) = self.target else {
                        return Flow::Done;
                    };
                    {
                        let pl = ctx.core.player_mut(t);
                        pl.hp = 1;
                        if pl.max_hp < 1 {
                            pl.max_hp = 1;
                        }
                        pl.death_cancelled = true;
                    }
                    Flow::Done
                }
                _ => Flow::Done,
            },
            // 遁行：确认 → SelectCard 弃一张 he → 判定非红桃 → 取消该牌
            //（cancelled 写回+桌面归还使用者手牌，工神同款）
            HostBehavior::SwdDunxing => match self.stage {
                HostStage::Act => {
                    let mut pool: Vec<CardId> =
                        ctx.core.player(self.owner).hand.iter().copied().collect();
                    pool.extend(ctx.core.player(self.owner).equips.iter().copied());
                    if pool.is_empty() {
                        return Flow::Done;
                    }
                    self.stage = HostStage::SelectCard;
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectCards,
                            prompt: "dunxing_discard",
                            options: pool.iter().copied().map(ChoiceOption::Card).collect(),
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
                    let in_hand = ctx.core.player(self.owner).hand.contains(&picked);
                    if in_hand {
                        ctx.take_hand_card(self.owner, picked);
                    } else {
                        ctx.core
                            .player_mut(self.owner)
                            .equips
                            .retain(|x| *x != picked);
                        if ctx.catalog.card_kind(picked).is_equip() {
                            crate::kernel::revoke_equip_skills(
                                ctx.core,
                                ctx.catalog,
                                self.owner,
                                picked,
                            );
                        }
                    }
                    ctx.to_discard(picked);
                    ctx.log(LogEvent::CardDiscarded {
                        player: self.owner,
                        cards: vec![picked],
                    });
                    let judged = ctx.draw_judge_card();
                    let heart = matches!(
                        ctx.catalog.cards[judged as usize].suit,
                        crate::catalog::Suit::Heart
                    );
                    ctx.to_discard(judged);
                    if !heart {
                        let (used, user) = {
                            let u = ctx.current_card_use_mut();
                            (u.card, u.player)
                        };
                        ctx.current_card_use_mut().cancelled = true;
                        ctx.core.table.retain(|x| *x != used);
                        ctx.core.player_mut(user).hand.push(used);
                    }
                    Flow::Done
                }
                _ => Flow::Done,
            },
            // 玉鳞：Act 问选装备牌（手/装备区）→ SelectCard 弃置+免伤；
            // 取消=不免伤（照常伤害）
            HostBehavior::SwdYulin => match self.stage {
                HostStage::Act => {
                    let options: Vec<ChoiceOption> = ctx
                        .core
                        .player(self.owner)
                        .hand
                        .iter()
                        .filter(|c| ctx.catalog.card_slot_of(ctx.core, **c).is_some())
                        .chain(ctx.core.player(self.owner).equips.iter())
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
                            prompt: "yulin_discard",
                            options,
                            min: 0,
                            max: 1,
                            allow_cancel: true,
                        },
                    }
                }
                HostStage::SelectCard => {
                    let picked: Option<CardId> = ctx.answer().iter().find_map(|o| match o {
                        ChoiceOption::Card(c) => Some(*c),
                        _ => None,
                    });
                    let Some(discarded) = picked else {
                        // 取消：伤害照常
                        return Flow::Done;
                    };
                    if ctx.core.player(self.owner).hand.contains(&discarded) {
                        ctx.discard_from_hand(self.owner, discarded);
                    } else {
                        ctx.core
                            .player_mut(self.owner)
                            .equips
                            .retain(|x| *x != discarded);
                        crate::kernel::revoke_equip_skills(
                            ctx.core,
                            ctx.catalog,
                            self.owner,
                            discarded,
                        );
                        ctx.to_discard(discarded);
                    }
                    ctx.log(LogEvent::CardDiscarded {
                        player: self.owner,
                        cards: vec![discarded],
                    });
                    ctx.current_damage_mut().prevented = true;
                    Flow::Done
                }
                _ => Flow::Done,
            },
            // 凝霜：Act 问选黑牌 → SelectCard 弃置+使用者阶梯
            // （已翻面→失 1 体力；已横置→翻面；否则→横置且自己摸 1）
            HostBehavior::SwdNingshuang => match self.stage {
                HostStage::Act => {
                    let is_black = |c: CardId| {
                        matches!(
                            ctx.catalog.cards[c as usize].suit,
                            crate::catalog::Suit::Spade | crate::catalog::Suit::Club
                        )
                    };
                    let options: Vec<ChoiceOption> = ctx
                        .core
                        .player(self.owner)
                        .hand
                        .iter()
                        .chain(ctx.core.player(self.owner).equips.iter())
                        .copied()
                        .filter(|c| is_black(*c))
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
                            prompt: "ningshuang_discard",
                            options,
                            min: 0,
                            max: 1,
                            allow_cancel: true,
                        },
                    }
                }
                HostStage::SelectCard => {
                    let picked: Option<CardId> = ctx.answer().iter().find_map(|o| match o {
                        ChoiceOption::Card(c) => Some(*c),
                        _ => None,
                    });
                    let Some(discarded) = picked else {
                        return Flow::Done;
                    };
                    if ctx.core.player(self.owner).hand.contains(&discarded) {
                        ctx.discard_from_hand(self.owner, discarded);
                    } else {
                        ctx.core
                            .player_mut(self.owner)
                            .equips
                            .retain(|x| *x != discarded);
                        crate::kernel::revoke_equip_skills(
                            ctx.core,
                            ctx.catalog,
                            self.owner,
                            discarded,
                        );
                        ctx.to_discard(discarded);
                    }
                    ctx.log(LogEvent::CardDiscarded {
                        player: self.owner,
                        cards: vec![discarded],
                    });
                    let Some(t) = self.target else {
                        return Flow::Done;
                    };
                    if !ctx.core.player(t).alive {
                        return Flow::Done;
                    }
                    if ctx.core.player(t).face_down {
                        // 已翻面：失 1 体力（不摸牌）
                        ctx.emit(EventData::LoseHp {
                            player: t,
                            amount: 1,
                        });
                    } else if ctx.core.player(t).linked {
                        // 已横置：翻面（不摸牌）
                        let flipped = {
                            let p = ctx.core.player_mut(t);
                            p.face_down = !p.face_down;
                            p.face_down
                        };
                        ctx.emit(EventData::TurnOvered {
                            player: t,
                            face_down: flipped,
                        });
                    } else {
                        // 横置（仅此支自己摸 1，上游代码原样）
                        ctx.emit(EventData::Linked {
                            player: t,
                            linked: true,
                        });
                        ctx.draw_cards(self.owner, 1);
                    }
                    Flow::Done
                }
                _ => Flow::Done,
            },
            // 工神：Act 问选装备牌 → SelectCard 弃置+取消该使用（筑城
            // cancelled 写回同款；取消=使用照常）
            HostBehavior::SwdGongshen => match self.stage {
                HostStage::Act => {
                    let options: Vec<ChoiceOption> = ctx
                        .core
                        .player(self.owner)
                        .hand
                        .iter()
                        .filter(|c| ctx.catalog.card_slot_of(ctx.core, **c).is_some())
                        .chain(ctx.core.player(self.owner).equips.iter())
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
                            prompt: "gongshen_discard",
                            options,
                            min: 0,
                            max: 1,
                            allow_cancel: true,
                        },
                    }
                }
                HostStage::SelectCard => {
                    let picked: Option<CardId> = ctx.answer().iter().find_map(|o| match o {
                        ChoiceOption::Card(c) => Some(*c),
                        _ => None,
                    });
                    let Some(discarded) = picked else {
                        return Flow::Done;
                    };
                    if ctx.core.player(self.owner).hand.contains(&discarded) {
                        ctx.discard_from_hand(self.owner, discarded);
                    } else {
                        ctx.core
                            .player_mut(self.owner)
                            .equips
                            .retain(|x| *x != discarded);
                        crate::kernel::revoke_equip_skills(
                            ctx.core,
                            ctx.catalog,
                            self.owner,
                            discarded,
                        );
                        ctx.to_discard(discarded);
                    }
                    ctx.log(LogEvent::CardDiscarded {
                        player: self.owner,
                        cards: vec![discarded],
                    });
                    {
                        let (used, user) = {
                            let u = ctx.current_card_use_mut();
                            (u.card, u.player)
                        };
                        ctx.current_card_use_mut().cancelled = true;
                        // 取消的使用：牌从桌面归还使用者手牌（上游
                        // trigger.cancel 同口径；提交段按 cancelled 跳过）
                        ctx.core.table.retain(|x| *x != used);
                        ctx.core.player_mut(user).hand.push(used);
                    }
                    Flow::Done
                }
                _ => Flow::Done,
            },
            _ => unreachable!("behavior routed to the wrong step module"),
        }
    }
}
