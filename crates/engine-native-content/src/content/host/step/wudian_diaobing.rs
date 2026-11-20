//! 武典（wudian）U07 宿主技/牌：筹度（choudu）与 调兵遣将（diaobingqianjiang）。

use super::super::*;
use super::wudian_cards::discard_owned_card_once;

impl HostMachine {
    pub(super) fn step_wudian_diaobing(&mut self, ctx: &mut Ctx<'_>) -> Flow {
        match self.behavior {
            HostBehavior::WudianChoudu => match self.stage {
                HostStage::Act => {
                    let p = ctx.core.player(self.owner);
                    let mut options: Vec<ChoiceOption> = p
                        .hand
                        .iter()
                        .chain(p.equips.iter())
                        .copied()
                        .map(ChoiceOption::Card)
                        .collect();
                    options.dedup();
                    if options.is_empty() {
                        return Flow::Done;
                    }
                    self.stage = HostStage::SelectCard;
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectCards,
                            prompt: WUDIAN_CHOUDU_CARD,
                            options,
                            min: 1,
                            max: 1,
                            allow_cancel: true,
                        },
                    }
                }
                HostStage::SelectCard => {
                    let Some(ChoiceOption::Card(c)) = ctx.answer().first().copied() else {
                        return Flow::Done;
                    };
                    self.card = Some(c);
                    let options: Vec<ChoiceOption> =
                        wudian_choudu_targets(ctx.core, ctx.catalog)
                            .into_iter()
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
                            prompt: WUDIAN_CHOUDU_TARGET,
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
                    let Some(c) = self.card.take() else {
                        return Flow::Done;
                    };
                    if !wudian_choudu_targets(ctx.core, ctx.catalog).contains(&t) {
                        return Flow::Done;
                    }
                    // 筹度（skill.js:587-600）：每出牌阶段限 1 次（vm_aux.once scope=4），
                    // 仅在真正提交发动时记次；按座次构建调兵遣将完整合法目标队列。
                    ctx.core.vm_aux.once.insert((
                        self.owner,
                        crate::content::host::WUDIAN_CHOUDU_KEY.to_string(),
                        4,
                    ));
                    if !discard_owned_card_once(ctx, self.owner, c) {
                        return Flow::Done;
                    }
                    ctx.emit(EventData::CardsDiscarded {
                        player: self.owner,
                        cards: vec![c],
                    });
                    // 上游 `await player.discard(cards); await player.useCard({ name: "diaobingqianjiang" })`：
                    // 先让 CardsDiscarded（如夺气 duoqi 等弃牌响应）完整结算，再于 DrawSettle 发射虚拟调兵遣将。
                    self.target = Some(t);
                    self.targets = ctx
                        .core
                        .seat_order_from(t)
                        .into_iter()
                        .filter(|&p| {
                            ctx.core.player(p).alive
                                && (p == t || !ctx.core.player(p).hand.is_empty())
                        })
                        .collect();
                    self.stage = HostStage::DrawSettle;
                    Flow::Yield
                }
                HostStage::DrawSettle => {
                    let Some(t) = self.target.take() else {
                        return Flow::Done;
                    };
                    if !ctx.core.player(t).alive {
                        return Flow::Done;
                    }
                    let mut targets = std::mem::take(&mut self.targets);
                    targets.retain(|&p| {
                        ctx.core.player(p).alive && (p == t || !ctx.core.player(p).hand.is_empty())
                    });
                    ctx.core
                        .vm_aux
                        .flags
                        .insert((t, "virtual_diaobing_use".to_string()), 1);
                    ctx.emit(EventData::CardUsed(crate::event::CardUseData {
                        card: None,
                        player: t,
                        targets,
                        nature: crate::event::DamageNature::Normal,
                        use_as: Some(CardKind::Wuzhong),
                        cancelled: false,
                    }));
                    Flow::Done
                }
                _ => Flow::Done,
            },
            // 调兵遣将（玩点论杀/card/card.js:4-142）：
            // Act: 使用者摸 1 -> DrawSettle: 亮出牌堆顶 ceil(alive/2) 张市场牌 ->
            // JiefenTarget (NullifyWindow): 每目标结算前经 _wuxie 无懈窗口 ->
            // SelectCard: 逐目标可选市场牌 (allow_cancel: true) ->
            // CardsAsk: 若选定则强制以 1 张手牌替换 (allow_cancel: false) ->
            // TargetChoice (contentAfter): 末尾使用者按选定顺序置任意张于牌堆顶、其余弃置。
            HostBehavior::WudianDiaobingCard => match self.stage {
                HostStage::Act => {
                    if !ctx.core.player(self.owner).alive {
                        self.diaobing_finish(ctx);
                        return Flow::Done;
                    }
                    ctx.start_draw(self.owner, 1);
                    self.stage = HostStage::DrawSettle;
                    Flow::Yield
                }
                HostStage::DrawSettle => {
                    if !ctx.core.player(self.owner).alive {
                        self.diaobing_finish(ctx);
                        return Flow::Done;
                    }
                    if matches!(ctx.core.mode, crate::state::GameMode::Versus { .. }) {
                        self.stage = HostStage::SelectNature;
                        return Flow::WaitChoice {
                            observer: self.owner,
                            spec: ChoiceSpec {
                                kind: ChoiceKind::PickTag,
                                prompt: "wudian_diaobing_dir",
                                options: vec![ChoiceOption::Tag(0), ChoiceOption::Tag(1)],
                                min: 1,
                                max: 1,
                                allow_cancel: false,
                            },
                        };
                    }
                    diaobing_reveal_and_start(self, ctx)
                }
                HostStage::SelectNature => {
                    if matches!(ctx.answer().first(), Some(ChoiceOption::Tag(0))) {
                        self.targets.reverse();
                        if let Some(pos) = self.targets.iter().position(|&p| p == self.owner) {
                            self.targets.rotate_left(pos);
                        }
                    }
                    diaobing_reveal_and_start(self, ctx)
                }
                HostStage::JiefenTarget => {
                    if ctx
                        .core
                        .vm_aux
                        .flags
                        .contains_key(&(self.owner, "diaobing_wuxie_active".to_string()))
                    {
                        return Flow::Yield;
                    }
                    let nullified = ctx
                        .core
                        .vm_aux
                        .flags
                        .remove(&(self.owner, "diaobing_wuxie_nullified".to_string()))
                        .is_some();
                    if nullified {
                        self.target = None;
                        return self.diaobing_next_target_or_after(ctx);
                    }
                    let Some(next_t) = self.target else {
                        return self.diaobing_next_target_or_after(ctx);
                    };
                    if self.cards.is_empty()
                        || !ctx.core.player(next_t).alive
                        || ctx.core.player(next_t).hand.is_empty()
                    {
                        self.target = None;
                        return self.diaobing_next_target_or_after(ctx);
                    }
                    self.stage = HostStage::SelectCard;
                    let options: Vec<ChoiceOption> =
                        self.cards.iter().copied().map(ChoiceOption::Card).collect();
                    Flow::WaitChoice {
                        observer: next_t,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectCards,
                            prompt: "wudian_diaobing_pick",
                            options,
                            min: 1,
                            max: 1,
                            allow_cancel: true,
                        },
                    }
                }
                HostStage::SelectCard => {
                    let Some(t) = self.target else {
                        return self.diaobing_next_target_or_after(ctx);
                    };
                    if let Some(ChoiceOption::Card(market_c)) = ctx.answer().first().copied() {
                        if self.cards.contains(&market_c)
                            && ctx.core.player(t).alive
                            && !ctx.core.player(t).hand.is_empty()
                        {
                            self.cards.retain(|&x| x != market_c);
                            self.cards.insert(0, market_c);
                            let hand_opts: Vec<ChoiceOption> = ctx
                                .core
                                .player(t)
                                .hand
                                .iter()
                                .copied()
                                .map(ChoiceOption::Card)
                                .collect();
                            self.stage = HostStage::CardsAsk;
                            return Flow::WaitChoice {
                                observer: t,
                                spec: ChoiceSpec {
                                    kind: ChoiceKind::SelectCards,
                                    prompt: WUDIAN_DIAOBING_SWAP,
                                    options: hand_opts,
                                    min: 1,
                                    max: 1,
                                    allow_cancel: false,
                                },
                            };
                        }
                    }
                    self.target = None;
                    self.diaobing_next_target_or_after(ctx)
                }
                HostStage::CardsAsk => {
                    if let Some(t) = self.target {
                        if let Some(ChoiceOption::Card(hand_c)) = ctx.answer().first().copied() {
                            if ctx.core.player(t).hand.contains(&hand_c) && !self.cards.is_empty() {
                                // 上游 card/card.js:87-90：先 lose（CardsLost 结算期间 market_c 尚未入手），
                                // 保持 market_c 在 cards[0]、hand_c 暂存 cards 末尾以守恒全区牌归属。
                                ctx.take_hand_card(t, hand_c);
                                self.cards.push(hand_c);
                                ctx.note_cards_lost(t, &[hand_c]);
                                ctx.emit(EventData::CardsLost {
                                    player: t,
                                    cards: vec![hand_c],
                                    origins: vec![crate::event::LostOrigin::Hand],
                                });
                                self.stage = HostStage::JiefenTargetDiscard;
                                return Flow::Yield;
                            }
                        }
                    }
                    self.target = None;
                    self.diaobing_next_target_or_after(ctx)
                }
                HostStage::JiefenTargetDiscard => {
                    if let Some(t) = self.target.take() {
                        if ctx.core.player(t).alive && !self.cards.is_empty() {
                            let market_c = self.cards.remove(0);
                            ctx.core.player_mut(t).hand.push(market_c);
                        }
                    }
                    self.diaobing_next_target_or_after(ctx)
                }
                HostStage::TargetChoice => {
                    let mut picked: Vec<CardId> = Vec::new();
                    for opt in ctx.answer() {
                        if let ChoiceOption::Card(cid) = *opt {
                            if self.cards.contains(&cid) && !picked.contains(&cid) {
                                picked.push(cid);
                            }
                        }
                    }
                    self.cards.retain(|c| !picked.contains(c));
                    // 先选择的在牌堆最顶（deck.pop 从末尾抽取）
                    for &cid in picked.iter().rev() {
                        ctx.core.deck.push(cid);
                    }
                    self.diaobing_finish(ctx);
                    Flow::Done
                }
                _ => Flow::Done,
            },
            _ => unreachable!("behavior routed to the wrong step module"),
        }
    }

    fn diaobing_next_target_or_after(&mut self, ctx: &mut Ctx<'_>) -> Flow {
        while !self.targets.is_empty() && !self.cards.is_empty() {
            let next_t = self.targets.remove(0);
            if ctx.core.player(next_t).alive && !ctx.core.player(next_t).hand.is_empty() {
                self.target = Some(next_t);
                self.stage = HostStage::JiefenTarget;
                let wuxie_card = self.card.unwrap_or_else(|| {
                    ctx.catalog
                        .cards
                        .iter()
                        .position(|d| d.name.contains("diaobingqianjiang"))
                        .unwrap_or(0) as CardId
                });
                ctx.core
                    .vm_aux
                    .flags
                    .insert((self.owner, "diaobing_wuxie_active".to_string()), 1);
                ctx.res.parks.insert(
                    0,
                    crate::resolution::Park {
                        resume_depth: ctx.res.stack.len(),
                        slot: crate::resolution::Slot::Intrinsic {
                            frame: crate::resolution::NO_FRAME,
                        },
                        machine: Box::new(crate::kernel::NullifyWindowMachine::new(
                            wuxie_card,
                            self.owner,
                            false,
                            ctx.core.seat_order_from(self.owner),
                            false,
                            true,
                            false,
                            None,
                        )),
                        awaiting_answer: false,
                    },
                );
                return Flow::Yield;
            }
        }
        self.target = None;
        if self.cards.is_empty() || !ctx.core.player(self.owner).alive {
            self.diaobing_finish(ctx);
            return Flow::Done;
        }
        self.stage = HostStage::TargetChoice;
        let options: Vec<ChoiceOption> =
            self.cards.iter().copied().map(ChoiceOption::Card).collect();
        Flow::WaitChoice {
            observer: self.owner,
            spec: ChoiceSpec {
                kind: ChoiceKind::SelectCards,
                prompt: "wudian_diaobing_top",
                options,
                min: 1,
                max: self.cards.len(),
                allow_cancel: true,
            },
        }
    }

    fn diaobing_finish(&mut self, ctx: &mut Ctx<'_>) {
        for rem in self.cards.drain(..) {
            ctx.to_discard(rem);
        }
        if let Some(c) = self.card.take() {
            if ctx.core.table.contains(&c) {
                ctx.core.table.retain(|&x| x != c);
                ctx.to_discard(c);
            }
        }
    }
}

fn diaobing_reveal_and_start(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let reveal_n = ctx.core.alive_players().count().div_ceil(2).max(1);
    m.cards.clear();
    for _ in 0..reveal_n {
        if let Some(cid) = ctx.core.deck.pop() {
            m.cards.push(cid);
        }
    }
    ctx.core.vm_aux.flags.insert(
        (m.owner, "u07_diaobing_resolved".to_string()),
        m.cards.len() as u32,
    );
    ctx.core
        .vm_aux
        .flags
        .insert((m.owner, "diaobingqianjiang_resolved".to_string()), 1);
    m.targets.retain(|p| ctx.core.player(*p).alive);
    if m.targets.is_empty() && ctx.core.player(m.owner).alive {
        m.targets.push(m.owner);
    }
    m.diaobing_next_target_or_after(ctx)
}

/// 筹度 `filterTarget: cardEnabled({ name: "diaobingqianjiang" }, target)`（Step 与 Asks 共用）。
pub(in crate::content::host) fn wudian_choudu_targets(
    core: &crate::state::GameStateCore,
    catalog: &crate::catalog::Catalog,
) -> Vec<PlayerId> {
    let db_cid = catalog
        .cards
        .iter()
        .position(|c| c.name.contains("diaobingqianjiang"))
        .map_or(u32::MAX, |i| i as crate::ids::CardId);
    core.alive_players()
        .filter(|&t| {
            !catalog.blocks_all_cards(core, t)
                && !core.player(t).jileng_active
                && !catalog
                    .skill_by_name("yudun")
                    .is_some_and(|d| core.has_skill(t, d))
                && !(t == core.turn.current
                    && core.player(t).anwugu_count > 0
                    && core.turn.cards_used_play_phase >= 3)
                && !crate::rvm_bridge::modifiers::eval_card_use_canceled(
                    core,
                    catalog,
                    t,
                    db_cid,
                    CardKind::Wuzhong,
                    crate::event::DamageNature::Normal,
                )
        })
        .collect()
}
