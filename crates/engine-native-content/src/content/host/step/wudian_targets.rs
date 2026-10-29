use super::super::*;

impl HostMachine {
    pub(super) fn step_wudian_targets(&mut self, ctx: &mut Ctx<'_>) -> Flow {
        match self.behavior {
            HostBehavior::WudianBaoxue => match self.stage {
                // 暴雪（U05 ow 美，规则 v127）：选至多「黑色 he 牌数」
                // 名其他角色（可整体放弃=limited 未消耗）→ 弃全部黑色
                // he 牌 → 逐目标其 he 区随机弃 1 张+翻至背面 → 自己翻
                // 至背面（turnOver(true) 幂等同口径）。登记差异：
                // showHandcards 表现层不建模；翻面内联无窗口。
                HostStage::SelectCard => {
                    let black = {
                        let p = ctx.core.player(self.owner);
                        p.hand
                            .iter()
                            .chain(p.equips.iter())
                            .filter(|c| {
                                matches!(
                                    ctx.catalog.cards[**c as usize].suit,
                                    crate::catalog::Suit::Spade | crate::catalog::Suit::Club
                                )
                            })
                            .count()
                    };
                    let options: Vec<ChoiceOption> = ctx
                        .core
                        .alive_players()
                        .filter(|p| *p != self.owner)
                        .map(ChoiceOption::Player)
                        .collect();
                    if options.is_empty() || black == 0 {
                        return Flow::Done;
                    }
                    self.stage = HostStage::TargetChoice;
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectPlayers,
                            prompt: WUDIAN_BAOXUE_TARGETS,
                            options,
                            min: 1,
                            max: black,
                            allow_cancel: true,
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
                        return Flow::Done; // 放弃：limited 未消耗
                    }
                    ctx.core.player_mut(self.owner).baoxue_used = true;
                    // contentBefore：弃全部黑色 he 牌
                    let black_cards: Vec<CardId> = {
                        let p = ctx.core.player(self.owner);
                        p.hand
                            .iter()
                            .copied()
                            .chain(p.equips.iter().copied())
                            .filter(|c| {
                                matches!(
                                    ctx.catalog.cards[*c as usize].suit,
                                    crate::catalog::Suit::Spade | crate::catalog::Suit::Club
                                )
                            })
                            .collect()
                    };
                    for c in &black_cards {
                        ctx.take_owned_card(self.owner, *c);
                        ctx.to_discard(*c);
                    }
                    ctx.log(LogEvent::CardDiscarded {
                        player: self.owner,
                        cards: black_cards.clone(),
                    });
                    // content：逐目标 he 区随机弃 1 张+翻至背面
                    for t in targets {
                        let pool: Vec<CardId> = {
                            let p = ctx.core.player(t);
                            p.hand
                                .iter()
                                .copied()
                                .chain(p.equips.iter().copied())
                                .collect()
                        };
                        if !pool.is_empty() {
                            let j = ctx.core.rng.gen_range(pool.len() as u32) as usize;
                            let c = pool[j];
                            ctx.take_owned_card(t, c);
                            ctx.to_discard(c);
                            ctx.log(LogEvent::CardDiscarded {
                                player: t,
                                cards: vec![c],
                            });
                        }
                        let flipped = {
                            let p = ctx.core.player_mut(t);
                            let f = !p.face_down;
                            p.face_down = true;
                            f
                        };
                        if flipped {
                            ctx.emit(EventData::TurnOvered {
                                player: t,
                                face_down: true,
                            });
                        }
                    }
                    // contentAfter：自己翻至背面（幂等）
                    let flipped = {
                        let p = ctx.core.player_mut(self.owner);
                        let f = !p.face_down;
                        p.face_down = true;
                        f
                    };
                    if flipped {
                        ctx.emit(EventData::TurnOvered {
                            player: self.owner,
                            face_down: true,
                        });
                    }
                    Flow::Done
                }
                _ => Flow::Done,
            },
            _ => unreachable!("behavior routed to the wrong step module"),
        }
    }
}
