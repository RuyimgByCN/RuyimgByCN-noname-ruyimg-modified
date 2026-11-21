use super::*;

fn finish_juxian_ban(
    ctx: &mut Ctx<'_>,
    owner: PlayerId,
    paid_red: bool,
    other: Option<CardId>,
) -> Flow {
    if let Some(c) = other {
        if paid_red != effective_is_red(ctx.core, ctx.catalog, owner, c) {
            set_turn_flag(ctx.core, owner, "dddjuxian_ban", "dddjuxian_stamp", 1);
        }
    }
    ctx.log(LogEvent::SkillTriggered {
        skill: "dddjuxian",
        owner,
        presentation: None,
    });
    Flow::Done
}

fn juxian_open_gain(machine: &mut HostMachine, ctx: &mut Ctx<'_>, _owner: PlayerId, paid: CardId, t: PlayerId) -> Flow {
    machine.target = Some(t);
    let mut shuffled = ctx.core.player(t).hand.clone();
    if shuffled.len() > 1 {
        ctx.core.rng.shuffle(&mut shuffled);
    }
    machine.cards = vec![paid];
    machine.cards.extend(shuffled);
    wait_ask_fix(machine, ctx, HostStage::CardsAsk, 1002)
}

pub(crate) fn step_a1(machine: &mut HostMachine, ctx: &mut Ctx<'_>) -> Option<Flow> {
    let owner = machine.owner;
    let flow = match machine.def_key.as_str() {
        // 1. `dddjuxian` (J01–J05, upstream `skill.js:244-383`)
        "dddjuxian" => match machine.stage {
            HostStage::Act => wait_ask_fix(machine, ctx, HostStage::SelectNature, 1001),
            HostStage::SelectNature => {
                let Some(ChoiceOption::Tag(mode)) = ctx.answer().first().copied() else {
                    return Some(Flow::Done);
                };
                machine.card = Some(u32::from(mode));
                wait_ask_fix(machine, ctx, HostStage::SelectCard, 1003)
            }
            HostStage::SelectCard => {
                let Some(ChoiceOption::Card(paid)) = ctx.answer().first().copied() else {
                    return Some(Flow::Done);
                };
                if !he_cards(ctx.core, owner).contains(&paid) {
                    return Some(Flow::Done);
                }
                machine.cards = vec![paid];
                machine.targets = vec![u8::from(effective_is_red(
                    ctx.core,
                    ctx.catalog,
                    owner,
                    paid,
                ))];
                if machine.card == Some(0) {
                    mark_scope4_used(ctx.core, owner, "dddjuxian_top");
                    ctx.log(LogEvent::CardsRevealed {
                        cards: vec![paid],
                        to: ctx.core.alive_players().collect(),
                    });
                    u01_remove_owned_ex(ctx, owner, paid);
                    ctx.core.deck.push(paid);
                    // J04: wait for `CardsLost` frame (and any sub-triggers) before target check!
                    machine.stage = HostStage::JiefenTarget;
                    Flow::Yield
                } else {
                    wait_ask_fix(machine, ctx, HostStage::TargetChoice, 1038)
                }
            }
            HostStage::JiefenTarget => {
                if machine.card == Some(1) {
                    // `hand` branch after `CardsLost`/`CardsMoved` settles: NOW snapshot pre_hand and start_draw!
                    if !ctx.core.player(owner).alive {
                        return Some(Flow::Done);
                    }
                    let paid = machine.cards.first().copied().unwrap_or(0);
                    let mut pre_hand = vec![paid];
                    pre_hand.extend(ctx.core.player(owner).hand.iter().copied());
                    machine.cards = pre_hand;
                    ctx.start_draw(owner, 1);
                    machine.stage = HostStage::DrawSettle;
                    return Some(Flow::Yield);
                }
                // `top` branch after `CardsLost` settles (J02 + J04)
                if !ctx.core.player(owner).alive {
                    return Some(Flow::Done);
                }
                let paid = machine.cards.first().copied().unwrap_or(0);
                let all_with_he: Vec<PlayerId> = ctx
                    .core
                    .alive_players()
                    .filter(|&p| !he_cards(ctx.core, p).is_empty())
                    .collect();
                if all_with_he.is_empty() {
                    ctx.log(LogEvent::SkillTriggered {
                        skill: "dddjuxian",
                        owner,
                        presentation: None,
                    });
                    Flow::Done
                } else if all_with_he.len() == 1 {
                    juxian_open_gain(machine, ctx, owner, paid, all_with_he[0])
                } else {
                    wait_ask_fix(machine, ctx, HostStage::TargetChoice, 1004)
                }
            }
            HostStage::TargetChoice => {
                let paid = machine.cards.first().copied().unwrap_or(0);
                let Some(ChoiceOption::Player(t)) = ctx.answer().first().copied() else {
                    return Some(Flow::Done);
                };
                if !ctx.core.player(t).alive {
                    return Some(Flow::Done);
                }
                if machine.card == Some(0) {
                    juxian_open_gain(machine, ctx, owner, paid, t)
                } else {
                    mark_scope4_used(ctx.core, owner, "dddjuxian_hand");
                    ctx.log(LogEvent::CardsRevealed {
                        cards: vec![paid],
                        to: ctx.core.alive_players().collect(),
                    });
                    u01_give_card_ex(ctx, owner, t, paid, false);
                    machine.target = Some(t);
                    machine.stage = HostStage::JiefenTarget;
                    Flow::Yield
                }
            }
            HostStage::DrawSettle => {
                let paid_red = machine.targets.first().copied().unwrap_or(0) != 0;
                let pre = &machine.cards[1..];
                let drawn = ctx
                    .core
                    .player(owner)
                    .hand
                    .iter()
                    .copied()
                    .find(|c| !pre.contains(c));
                if let Some(d) = drawn {
                    ctx.log(LogEvent::CardsRevealed {
                        cards: vec![d],
                        to: ctx.core.alive_players().collect(),
                    });
                }
                finish_juxian_ban(ctx, owner, paid_red, drawn)
            }
            HostStage::CardsAsk => {
                let paid_red = machine.targets.first().copied().unwrap_or(0) != 0;
                let Some(t) = machine.target else {
                    return Some(Flow::Done);
                };
                let gained = match ctx.answer().first().copied() {
                    Some(ChoiceOption::HiddenCard(slot)) => machine
                        .cards
                        .get(1 + slot as usize)
                        .copied()
                        .filter(|c| ctx.core.player(t).hand.contains(c)),
                    Some(ChoiceOption::Card(c))
                        if (t == owner && ctx.core.player(t).hand.contains(&c))
                            || ctx.core.player(t).equips.contains(&c) =>
                    {
                        Some(c)
                    }
                    _ => None,
                };
                if let Some(c) = gained {
                    // Upstream `content.ts:11251`: `gainPlayerCard(owner, "he", true)` on own
                    // hand card fires zero `lose`/`CardsLost`/`CardsMoved` events; only own
                    // equips (moved to hand) or cards taken from another player emit lose/move.
                    if !(t == owner && ctx.core.player(owner).hand.contains(&c)) {
                        u01_give_card_ex(ctx, t, owner, c, false);
                    }
                }
                finish_juxian_ban(ctx, owner, paid_red, gained)
            }
            _ => Flow::Done,
        },
        // 2. `dddzhishu` (upstream `skill.js:37-58`)
        "dddzhishu" => match machine.stage {
            HostStage::Act => wait_ask_fix(machine, ctx, HostStage::SelectCard, 1005),
            HostStage::SelectCard => {
                let Some(ChoiceOption::Card(c)) = ctx.answer().first().copied() else {
                    return Some(Flow::Done);
                };
                let Some(src) = ctx
                    .core
                    .alive_players()
                    .find(|&p| ctx.core.player(p).equips.contains(&c) || ctx.core.player(p).judge.contains(&c))
                else {
                    return Some(Flow::Done);
                };
                machine.card = Some(c);
                machine.target = Some(src);
                wait_ask_fix(machine, ctx, HostStage::TargetChoice, 1006)
            }
            HostStage::TargetChoice => {
                let (Some(c), Some(src), Some(ChoiceOption::Player(dst))) =
                    (machine.card, machine.target, ctx.answer().first().copied())
                else {
                    return Some(Flow::Done);
                };
                if !ctx.core.player(src).alive || !ctx.core.player(dst).alive || src == dst {
                    return Some(Flow::Done);
                }
                if ctx.core.player(src).judge.contains(&c) {
                    let decl = ctx.core.declared_kinds.get(&c).copied();
                    u01_remove_owned_ex(ctx, src, c);
                    if let Some(k) = decl {
                        ctx.core.declared_kinds.insert(c, k);
                    }
                    ctx.core.player_mut(dst).judge.push(c);
                    ctx.emit(EventData::CardsMoved {
                        from: src,
                        to: dst,
                        cards: vec![c],
                    });
                } else if ctx.core.player(src).equips.contains(&c) {
                    let decl_slot = ctx.core.declared_slots.get(&c).copied();
                    u01_remove_owned_ex(ctx, src, c);
                    if let Some(slot) = decl_slot {
                        ctx.core.declared_slots.insert(c, slot);
                    }
                    ctx.core.player_mut(dst).equips.push(c);
                    crate::kernel::grant_equip_skills(ctx, dst, c);
                    ctx.emit(EventData::CardsMoved {
                        from: src,
                        to: dst,
                        cards: vec![c],
                    });
                }
                ctx.log(LogEvent::SkillTriggered {
                    skill: "dddzhishu",
                    owner,
                    presentation: None,
                });
                machine.target = Some(src);
                machine.targets = vec![dst];
                machine.stage = HostStage::DrawSettle;
                Flow::Yield
            }
            HostStage::DrawSettle => {
                let (Some(src), Some(dst)) = (machine.target, machine.targets.first().copied())
                else {
                    return Some(Flow::Done);
                };
                if !ctx.core.player(src).alive || !ctx.core.player(dst).alive {
                    return Some(Flow::Done);
                }
                let can_dismantle = !ctx.catalog.blocks_all_cards(ctx.core, src)
                    && !is_target_protected(ctx.core, dst)
                    && !crate::rvm_bridge::modifiers::eval_card_use_canceled(
                        ctx.core,
                        ctx.catalog,
                        src,
                        0,
                        CardKind::Dismantle,
                        DamageNature::Normal,
                    )
                    && !crate::rvm_bridge::modifiers::eval_card_target_canceled(
                        ctx.core,
                        ctx.catalog,
                        src,
                        dst,
                        0,
                        CardKind::Dismantle,
                        DamageNature::Normal,
                    )
                    && (!ctx.core.player(dst).hand.is_empty()
                        || !ctx.core.player(dst).equips.is_empty()
                        || !ctx.core.player(dst).judge.is_empty());
                if can_dismantle {
                    ctx.emit(EventData::CardUsed(CardUseData {
                        card: None,
                        player: src,
                        targets: vec![dst],
                        nature: DamageNature::Normal,
                        use_as: Some(CardKind::Dismantle),
                        cancelled: false,
                    }));
                }
                Flow::Done
            }
            _ => Flow::Done,
        },
        // 3. `dddjungui` (upstream `skill.js:384-441`)
        "dddjungui" => match machine.stage {
            HostStage::Act => wait_ask_fix(machine, ctx, HostStage::TargetChoice, 1014),
            HostStage::TargetChoice => {
                let Some(ChoiceOption::Player(t)) = ctx.answer().first().copied() else {
                    return Some(Flow::Done);
                };
                if !ctx.core.player(t).alive {
                    return Some(Flow::Done);
                }
                machine.target = Some(t);
                ctx.log(LogEvent::SkillTriggered {
                    skill: "dddjungui",
                    owner,
                    presentation: None,
                });
                ctx.start_draw(t, 2);
                machine.stage = HostStage::DrawSettle;
                Flow::Yield
            }
            HostStage::DrawSettle => {
                let Some(t) = machine.target else {
                    return Some(Flow::Done);
                };
                if !ctx.core.player(t).alive {
                    return Some(Flow::Done);
                }
                let hand = ctx.core.player(t).hand.clone();
                if !hand.is_empty() {
                    ctx.log(LogEvent::CardsRevealed {
                        cards: hand.clone(),
                        to: ctx.core.alive_players().collect(),
                    });
                }
                let target_suits = effective_distinct_suits(ctx.core, ctx.catalog, t, &hand);
                let used = turn_flag(ctx.core, owner, "dddjungui_suits_mask", "dddjungui_suits_stamp")
                    .count_ones() as usize;
                machine.cards.clear();
                machine.card = Some(target_suits as u32);
                if used > 0 {
                    wait_ask_fix(machine, ctx, HostStage::SelectNature, 1015)
                } else if target_suits == 0 {
                    Flow::Done
                } else {
                    wait_ask_fix(machine, ctx, HostStage::SelectCard, 1016)
                }
            }
            HostStage::SelectNature => {
                let reduce = match ctx.answer().first().copied() {
                    Some(ChoiceOption::Tag(r)) => r as usize,
                    _ => 0,
                };
                let target_suits = machine.card.unwrap_or(0) as usize;
                let rem = target_suits.saturating_sub(reduce);
                if rem == 0 {
                    return Some(Flow::Done);
                }
                machine.card = Some(rem as u32);
                wait_ask_fix(machine, ctx, HostStage::SelectCard, 1016)
            }
            HostStage::SelectCard => {
                let Some(t) = machine.target else {
                    return Some(Flow::Done);
                };
                let need = machine.card.unwrap_or(0) as usize;
                let picks: Vec<CardId> = ctx
                    .answer()
                    .iter()
                    .filter_map(|o| match o {
                        ChoiceOption::Card(c) => Some(*c),
                        _ => None,
                    })
                    .filter(|c| ctx.core.player(t).hand.contains(c))
                    .collect();
                if picks.len() != need
                    || need == 0
                    || effective_distinct_suits(ctx.core, ctx.catalog, t, &picks) != need
                {
                    return Some(Flow::Done);
                }
                u01_discard_batch(ctx, t, &picks);
                Flow::Done
            }
            _ => Flow::Done,
        },
        // 4. `dddyeshen` (upstream `skill.js:656-746`)
        "dddyeshen" => step_yeshen(machine, ctx),
        // 5. `dddqiaoduan` (upstream `skill.js:747-801`)
        "dddqiaoduan" => step_qiaoduan(machine, ctx),
        _ => return None,
    };
    Some(flow)
}

fn step_yeshen(machine: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = machine.owner;
    match machine.stage {
        HostStage::Act => wait_ask_fix(machine, ctx, HostStage::JiefenTarget, 1007),
        HostStage::JiefenTarget => {
            if !matches!(ctx.answer().first().copied(), Some(ChoiceOption::Confirm)) {
                return Flow::Done;
            }
            let Some(t) = machine.target else {
                return Flow::Done;
            };
            let mark = flag(ctx.core, owner, "dddyeshen_count").min(2) as usize;
            let take_n = 3usize.saturating_sub(mark);
            let mut drawn = Vec::new();
            for _ in 0..take_n {
                if ctx.core.deck.is_empty() && !ctx.core.discard.is_empty() {
                    let mut cards = std::mem::take(&mut ctx.core.discard);
                    ctx.core.rng.shuffle(&mut cards);
                    crate::kernel::elo_on_wash_deck(ctx.core, &cards);
                    ctx.core.deck = cards;
                    ctx.out.push(LogEvent::DeckReshuffled);
                }
                if !ctx.core.deck.is_empty() {
                    let c = ctx.core.deck.remove(0);
                    ctx.core.table.push(c);
                    drawn.push(c);
                }
            }
            if drawn.is_empty() {
                return yeshen_finish_mark(ctx, owner);
            }
            ctx.log(LogEvent::CardsRevealed {
                cards: drawn.clone(),
                to: ctx.core.alive_players().collect(),
            });
            machine.cards = drawn;
            let blacks: Vec<CardId> = machine
                .cards
                .iter()
                .copied()
                .filter(|&c| is_black(ctx.catalog, c))
                .collect();
            if blacks.is_empty() || !ctx.core.player(t).alive {
                yeshen_return_unused(machine, ctx, None);
                return yeshen_finish_mark(ctx, owner);
            }
            if blacks.len() > 1 {
                wait_ask_fix(machine, ctx, HostStage::SelectCard, 1008)
            } else {
                machine.card = Some(blacks[0]);
                yeshen_open_targets(machine, ctx, t, blacks[0])
            }
        }
        HostStage::SelectCard => {
            let Some(t) = machine.target else {
                yeshen_return_unused(machine, ctx, None);
                return yeshen_finish_mark(ctx, owner);
            };
            let Some(ChoiceOption::Card(picked)) = ctx.answer().first().copied() else {
                yeshen_return_unused(machine, ctx, None);
                return yeshen_finish_mark(ctx, owner);
            };
            machine.card = Some(picked);
            yeshen_open_targets(machine, ctx, t, picked)
        }
        HostStage::TargetChoice => {
            let Some(t) = machine.target else {
                yeshen_return_unused(machine, ctx, None);
                return yeshen_finish_mark(ctx, owner);
            };
            let Some(picked) = machine.card else {
                yeshen_return_unused(machine, ctx, None);
                return yeshen_finish_mark(ctx, owner);
            };
            let targets: Vec<PlayerId> = ctx
                .answer()
                .iter()
                .filter_map(|o| match o {
                    ChoiceOption::Player(p) => Some(*p),
                    _ => None,
                })
                .collect();
            if !targets.is_empty() && ctx.core.player(t).alive {
                ctx.emit(EventData::CardUsed(CardUseData {
                    card: Some(picked),
                    player: t,
                    targets,
                    nature: DamageNature::Normal,
                    use_as: Some(CardKind::Chain),
                    cancelled: false,
                }));
            } else {
                ctx.core.table.retain(|&x| x != picked);
                ctx.to_discard(picked);
                ctx.log(LogEvent::CardDiscarded {
                    player: t,
                    cards: vec![picked],
                });
                if ctx.core.player(t).alive {
                    ctx.start_draw(t, 1);
                }
            }
            machine.stage = HostStage::DrawSettle;
            Flow::Yield
        }
        HostStage::DrawSettle => {
            yeshen_return_unused(machine, ctx, machine.card);
            yeshen_finish_mark(ctx, owner)
        }
        _ => Flow::Done,
    }
}

fn yeshen_open_targets(
    machine: &mut HostMachine,
    ctx: &mut Ctx<'_>,
    t: PlayerId,
    picked: CardId,
) -> Flow {
    set_flag(ctx.core, machine.owner, ASK_KEY, 1009);
    if build_ask_fix(
        1009,
        machine.owner,
        Some(t),
        &machine.targets,
        Some(picked),
        &machine.cards,
        ctx.core,
        ctx.catalog,
    )
    .is_some()
    {
        wait_ask_fix(machine, ctx, HostStage::TargetChoice, 1009)
    } else {
        machine.card = Some(picked);
        ctx.core.table.retain(|&x| x != picked);
        ctx.to_discard(picked);
        if ctx.core.player(t).alive {
            ctx.start_draw(t, 1);
        }
        machine.stage = HostStage::DrawSettle;
        Flow::Yield
    }
}

fn yeshen_return_unused(machine: &mut HostMachine, ctx: &mut Ctx<'_>, used_card: Option<CardId>) {
    for c in machine.cards.drain(..) {
        if Some(c) == used_card {
            continue;
        }
        if ctx.core.table.contains(&c) {
            ctx.core.table.retain(|&x| x != c);
            ctx.core.deck.push(c);
        }
    }
}

fn yeshen_finish_mark(ctx: &mut Ctx<'_>, owner: PlayerId) -> Flow {
    let cur = flag(ctx.core, owner, "dddyeshen_count");
    if cur < 2 {
        set_flag(ctx.core, owner, "dddyeshen_count", cur + 1);
    } else {
        set_flag(ctx.core, owner, "dddyeshen_count", 0);
        if ctx.core.player(owner).alive {
            ctx.emit(EventData::Damage(damage_data_with(
                Some(owner),
                owner,
                1,
                DamageNature::Fire,
                None,
            )));
        }
    }
    ctx.log(LogEvent::SkillTriggered {
        skill: "dddyeshen",
        owner,
        presentation: None,
    });
    Flow::Done
}

fn step_qiaoduan(machine: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = machine.owner;
    match machine.stage {
        HostStage::Act => {
            let linked_cnt = ctx
                .core
                .alive_players()
                .filter(|&p| ctx.core.player(p).linked)
                .count();
            if linked_cnt == 0 {
                return Flow::Done;
            }
            let was_linked = machine.target.is_some_and(|t| ctx.core.player(t).linked);
            machine.card = Some(linked_cnt as u32);
            if was_linked {
                machine.targets = vec![1];
                wait_ask_fix(machine, ctx, HostStage::SelectCard, 1019)
            } else {
                machine.targets = vec![0];
                wait_ask_fix(machine, ctx, HostStage::TargetChoice, 1021)
            }
        }
        HostStage::SelectCard => {
            let cards: Vec<CardId> = ctx
                .answer()
                .iter()
                .filter_map(|o| match o {
                    ChoiceOption::Card(c) => Some(*c),
                    _ => None,
                })
                .filter(|c| ctx.core.player(owner).hand.contains(c))
                .collect();
            let need = machine.card.unwrap_or(0) as usize;
            if cards.len() != need || cards.is_empty() {
                return Flow::Done;
            }
            machine.cards = cards;
            wait_ask_fix(machine, ctx, HostStage::TargetChoice, 1020)
        }
        HostStage::TargetChoice => {
            let picked: Vec<PlayerId> = ctx
                .answer()
                .iter()
                .filter_map(|o| match o {
                    ChoiceOption::Player(p) => Some(*p),
                    _ => None,
                })
                .collect();
            if picked.is_empty() {
                return Flow::Done;
            }
            set_turn_flag(ctx.core, owner, "dddqiaoduan_used_turn", "dddqiaoduan_stamp", 1);
            if machine.targets.first().copied() == Some(1) {
                let cards = std::mem::take(&mut machine.cards);
                let mut lost = Vec::new();
                for c in cards {
                    if ctx.core.player(owner).hand.contains(&c) {
                        ctx.take_hand_card(owner, c);
                        ctx.core.deck.insert(0, c);
                        lost.push(c);
                    }
                }
                let n = lost.len();
                if !lost.is_empty() {
                    ctx.emit(EventData::CardsLost {
                        player: owner,
                        cards: lost,
                        origins: vec![LostOrigin::Hand; n],
                    });
                }
                machine.target = picked.first().copied();
                machine.stage = HostStage::DrawSettle;
                return Flow::Yield;
            } else {
                let order: Vec<PlayerId> = ctx
                    .core
                    .seat_order_from(ctx.core.turn.current)
                    .into_iter()
                    .filter(|p| picked.contains(p) && ctx.core.player(*p).alive)
                    .collect();
                ctx.log(LogEvent::SkillTriggered {
                    skill: "dddqiaoduan",
                    owner,
                    presentation: None,
                });
                for p in order.into_iter().rev() {
                    ctx.start_draw(p, 1);
                }
                Flow::Done
            }
        }
        HostStage::DrawSettle => {
            if let Some(t) = machine.target {
                if ctx.core.player(t).alive
                    && ctx.core.player(t).hp < ctx.core.player(t).max_hp as i32
                {
                    ctx.recover(t, 1, None);
                }
            }
            ctx.log(LogEvent::SkillTriggered {
                skill: "dddqiaoduan",
                owner,
                presentation: None,
            });
            Flow::Done
        }
        _ => Flow::Done,
    }
}
