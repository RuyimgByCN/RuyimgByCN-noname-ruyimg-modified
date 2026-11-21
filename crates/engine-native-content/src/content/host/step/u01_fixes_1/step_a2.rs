use super::*;

pub(crate) fn step_a2(machine: &mut HostMachine, ctx: &mut Ctx<'_>) -> Option<Flow> {
    let flow = match machine.def_key.as_str() {
        // 6. `ddddiedang` (upstream `skill.js:802-860`)
        "ddddiedang" => step_diedang(machine, ctx),
        // 7. `dddanliu` (upstream `skill.js:861-967`)
        "dddanliu" => step_anliu(machine, ctx),
        // 8. `dddguiying` (upstream `skill.js:968-1203`)
        "dddguiying" => step_guiying(machine, ctx),
        // 9. `dddshilie` (upstream `skill.js:511-654`)
        "dddshilie" => step_shilie(machine, ctx),
        // 10. `dddbingjian` (upstream `skill.js:1204-1441`)
        "dddbingjian" => step_bingjian(machine, ctx),
        // 11. `dddtongyu` (upstream `skill.js:1443-1583`)
        "dddtongyu" => step_tongyu(machine, ctx),
        _ => return None,
    };
    Some(flow)
}

fn step_diedang(machine: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = machine.owner;
    let st = flag(ctx.core, owner, "ddddiedang_state");
    match machine.stage {
        HostStage::Act => {
            if st == 0 {
                mark_scope4_used(ctx.core, owner, "ddddiedang");
                ctx.start_draw(owner, 3);
                machine.stage = HostStage::DrawSettle;
                Flow::Yield
            } else {
                wait_ask_fix(machine, ctx, HostStage::SelectCard, 1024)
            }
        }
        HostStage::SelectCard => {
            let picks: Vec<CardId> = ctx
                .answer()
                .iter()
                .filter_map(|o| match o {
                    ChoiceOption::Card(c) => Some(*c),
                    _ => None,
                })
                .collect();
            if st == 0 {
                if let Some(&c) = picks.first() {
                    u01_discard_one(ctx, owner, c);
                }
                machine.stage = HostStage::SelectNature;
                Flow::Yield
            } else {
                if picks.len() != 3 {
                    return Flow::Done;
                }
                mark_scope4_used(ctx.core, owner, "ddddiedang");
                u01_discard_batch(ctx, owner, &picks);
                machine.stage = HostStage::JiefenTarget;
                Flow::Yield
            }
        }
        HostStage::SelectNature => diedang_finish_toggle(ctx, owner, st),
        HostStage::JiefenTarget => {
            ctx.start_draw(owner, 1);
            machine.stage = HostStage::DrawSettle;
            Flow::Yield
        }
        HostStage::DrawSettle => {
            // Note: st == 0 uses DrawSettle for draw(3) settle BEFORE discarding; st == 1 uses CardsAsk? No! Wait! Check!
            if st == 0 {
                if he_cards(ctx.core, owner).is_empty() {
                    diedang_finish_toggle(ctx, owner, st)
                } else {
                    wait_ask_fix(machine, ctx, HostStage::SelectCard, 1023)
                }
            } else {
                diedang_finish_toggle(ctx, owner, st)
            }
        }
        _ => Flow::Done,
    }
}

fn diedang_finish_toggle(ctx: &mut Ctx<'_>, owner: PlayerId, st: u32) -> Flow {
    let h = ctx.core.player(owner).hand.len();
    let max = ctx
        .core
        .alive_players()
        .map(|p| ctx.core.player(p).hand.len())
        .max()
        .unwrap_or(0);
    let min = ctx
        .core
        .alive_players()
        .map(|p| ctx.core.player(p).hand.len())
        .min()
        .unwrap_or(0);
    if h == max || h == min {
        set_flag(ctx.core, owner, "ddddiedang_state", if st == 0 { 1 } else { 0 });
    }
    ctx.log(LogEvent::SkillTriggered {
        skill: "ddddiedang",
        owner,
        presentation: None,
    });
    Flow::Done
}

fn step_anliu(machine: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = machine.owner;
    match machine.stage {
        HostStage::Act => wait_ask_fix(machine, ctx, HostStage::TargetChoice, 1025),
        HostStage::TargetChoice => {
            let Some(ChoiceOption::Player(t)) = ctx.answer().first().copied() else {
                return Flow::Done;
            };
            if !ctx.core.player(t).alive || ctx.core.player(t).hand.is_empty() {
                return Flow::Done;
            }
            machine.target = Some(t);
            wait_ask_fix(machine, ctx, HostStage::SelectCard, 1026)
        }
        HostStage::SelectCard => {
            let Some(ChoiceOption::Card(c1)) = ctx.answer().first().copied() else {
                return Flow::Done;
            };
            let Some(t) = machine.target else {
                return Flow::Done;
            };
            if !ctx.core.player(t).hand.contains(&c1) {
                return Flow::Done;
            }
            machine.cards = vec![c1];
            wait_ask_fix(machine, ctx, HostStage::CardsAsk, 1027)
        }
        HostStage::CardsAsk => {
            let Some(ChoiceOption::Card(c2)) = ctx.answer().first().copied() else {
                return Flow::Done;
            };
            let (Some(t), Some(&c1)) = (machine.target, machine.cards.first()) else {
                return Flow::Done;
            };
            if !ctx.core.player(owner).hand.contains(&c2) {
                return Flow::Done;
            }
            machine.cards = vec![c1, c2];
            if effective_is_red(ctx.core, ctx.catalog, t, c1)
                != effective_is_red(ctx.core, ctx.catalog, owner, c2)
            {
                ctx.log(LogEvent::SkillTriggered {
                    skill: "dddanliu",
                    owner,
                    presentation: None,
                });
                return Flow::Done;
            }
            wait_ask_fix(machine, ctx, HostStage::JiefenTarget, 1028)
        }
        HostStage::JiefenTarget => {
            if !matches!(ctx.answer().first().copied(), Some(ChoiceOption::Confirm)) {
                return Flow::Done;
            }
            let (Some(t), Some(&c1), Some(&c2)) =
                (machine.target, machine.cards.first(), machine.cards.get(1))
            else {
                return Flow::Done;
            };
            if ctx.core.player(t).hand.contains(&c1) && ctx.core.player(owner).hand.contains(&c2) {
                let c1_red = effective_is_red(ctx.core, ctx.catalog, t, c1);
                machine.card = Some(u32::from(c1_red));
                u01_swap_handcards_simultaneous(ctx, t, &[c1], owner, &[c2]);
                machine.stage = HostStage::DrawSettle;
                return Flow::Yield;
            }
            ctx.log(LogEvent::SkillTriggered {
                skill: "dddanliu",
                owner,
                presentation: None,
            });
            Flow::Done
        }
        HostStage::DrawSettle => {
            if let Some(t) = machine.target {
                let c1_red = machine.card.unwrap_or(0) != 0;
                if c1_red {
                    if ctx.core.player(owner).alive
                        && ctx.core.player(owner).hp < ctx.core.player(owner).max_hp as i32
                    {
                        ctx.recover(owner, 1, None);
                    }
                } else if ctx.core.player(t).alive {
                    ctx.start_draw(t, 2);
                }
            }
            ctx.log(LogEvent::SkillTriggered {
                skill: "dddanliu",
                owner,
                presentation: None,
            });
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn guiying_next_wu(machine: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = machine.owner;
    while !machine.targets.is_empty() {
        let next = machine.targets.remove(0);
        if next != owner
            && ctx.core.player(next).alive
            && ctx.core.effective_faction(ctx.catalog, next) == Some(crate::catalog::Faction::Wu)
            && !ctx.core.player(next).hand.is_empty()
        {
            machine.target = Some(next);
            return wait_ask_fix(machine, ctx, HostStage::JiefenTarget, 1010);
        }
    }
    Flow::Done
}

fn guiying_after_give(
    machine: &mut HostMachine,
    ctx: &mut Ctx<'_>,
    owner: PlayerId,
    t: PlayerId,
    c: CardId,
) -> Flow {
    if he_cards(ctx.core, t).contains(&c) {
        u01_give_card_ex(ctx, t, owner, c, true);
        machine.stage = HostStage::DrawSettle;
        return Flow::Yield;
    }
    if asks_a::dddguiying_min_ej_rank(ctx.core, ctx.catalog).is_some() && ctx.core.player(t).alive {
        wait_ask_fix(machine, ctx, HostStage::TargetChoice, 1012)
    } else {
        guiying_next_wu(machine, ctx)
    }
}

fn step_guiying(machine: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = machine.owner;
    match machine.stage {
        HostStage::Act => {
            machine.targets = ctx
                .core
                .seat_order_from(owner)
                .into_iter()
                .filter(|&p| p != owner)
                .collect();
            guiying_next_wu(machine, ctx)
        }
        HostStage::JiefenTarget => {
            if !matches!(ctx.answer().first().copied(), Some(ChoiceOption::Confirm)) {
                return guiying_next_wu(machine, ctx);
            }
            let Some(t) = machine.target else {
                return guiying_next_wu(machine, ctx);
            };
            ctx.log(LogEvent::SkillTriggered {
                skill: "dddguiying",
                owner: t,
                presentation: None,
            });
            let hand = ctx.core.player(t).hand.clone();
            if !hand.is_empty() {
                ctx.log(LogEvent::CardsRevealed {
                    cards: hand,
                    to: ctx.core.alive_players().collect(),
                });
            }
            let max_cands = asks_a::dddguiying_max_he_cards(ctx.core, ctx.catalog, t);
            if max_cands.len() > 1 {
                wait_ask_fix(machine, ctx, HostStage::SelectCard, 1011)
            } else if let Some(&c) = max_cands.first() {
                guiying_after_give(machine, ctx, owner, t, c)
            } else {
                guiying_next_wu(machine, ctx)
            }
        }
        HostStage::SelectCard => {
            let Some(t) = machine.target else {
                return guiying_next_wu(machine, ctx);
            };
            let Some(ChoiceOption::Card(c)) = ctx.answer().first().copied() else {
                return guiying_next_wu(machine, ctx);
            };
            guiying_after_give(machine, ctx, owner, t, c)
        }
        HostStage::TargetChoice => {
            let Some(t) = machine.target else {
                return guiying_next_wu(machine, ctx);
            };
            let Some(ChoiceOption::Player(p2)) = ctx.answer().first().copied() else {
                return guiying_next_wu(machine, ctx);
            };
            let cands = asks_a::dddguiying_min_ej_of(ctx.core, ctx.catalog, p2);
            if cands.len() > 1 {
                machine.card = Some(u32::from(p2));
                wait_ask_fix(machine, ctx, HostStage::CardsAsk, 1013)
            } else if let Some(&ej) = cands.first() {
                u01_give_card_ex(ctx, p2, t, ej, true);
                machine.stage = HostStage::SelectNature;
                Flow::Yield
            } else {
                guiying_next_wu(machine, ctx)
            }
        }
        HostStage::CardsAsk => {
            let Some(t) = machine.target else {
                return guiying_next_wu(machine, ctx);
            };
            if let (Some(p2), Some(ChoiceOption::Card(ej))) =
                (machine.card.map(|v| v as PlayerId), ctx.answer().first().copied())
            {
                u01_give_card_ex(ctx, p2, t, ej, true);
                machine.stage = HostStage::SelectNature;
                return Flow::Yield;
            }
            guiying_next_wu(machine, ctx)
        }
        HostStage::DrawSettle => {
            let Some(t) = machine.target else {
                return guiying_next_wu(machine, ctx);
            };
            if asks_a::dddguiying_min_ej_rank(ctx.core, ctx.catalog).is_some()
                && ctx.core.player(t).alive
            {
                wait_ask_fix(machine, ctx, HostStage::TargetChoice, 1012)
            } else {
                guiying_next_wu(machine, ctx)
            }
        }
        HostStage::SelectNature => guiying_next_wu(machine, ctx),
        _ => Flow::Done,
    }
}

fn finish_respond_park(ctx: &mut Ctx<'_>, owner: PlayerId, ok: bool) -> Flow {
    let fid = flag(ctx.core, owner, "u01_respond_fid");
    if fid != 0 {
        set_flag(ctx.core, owner, &format!("u01_respond_result:{fid}"), u32::from(ok));
    }
    Flow::Done
}

fn step_shilie(machine: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = machine.owner;
    let is_defensive = machine.card == Some(10);
    match machine.stage {
        HostStage::Act => wait_ask_fix(machine, ctx, HostStage::SelectCard, 1017),
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
                return if is_defensive {
                    finish_respond_park(ctx, owner, false)
                } else {
                    Flow::Done
                };
            }
            let sum: i32 = picked
                .iter()
                .map(|&c| i32::from(ctx.catalog.cards[c as usize].rank))
                .sum();
            let threshold = asks_a::dddshilie_threshold(ctx.core, owner);
            if sum < threshold {
                return if is_defensive {
                    finish_respond_park(ctx, owner, false)
                } else {
                    Flow::Done
                };
            }
            machine.cards = picked;
            machine.targets = vec![u8::from(sum == threshold)];
            if is_defensive {
                set_turn_flag(ctx.core, owner, "dddshilie_used_turn", "dddshilie_stamp", 1);
                for &c in &machine.cards {
                    set_flag(ctx.core, owner, &format!("dddshilie_shown_card:{c}"), 1);
                }
                ctx.log(LogEvent::CardsRevealed {
                    cards: machine.cards.clone(),
                    to: ctx.core.alive_players().collect(),
                });
                ctx.log(LogEvent::SkillTriggered {
                    skill: "dddshilie",
                    owner,
                    presentation: None,
                });
                if sum == threshold {
                    ctx.start_draw(owner, machine.cards.len() as u32);
                    machine.stage = HostStage::JiefenTarget;
                    return Flow::Yield;
                }
                ctx.emit(EventData::CardUsed(CardUseData {
                    card: None,
                    player: owner,
                    targets: Vec::new(),
                    nature: DamageNature::Normal,
                    use_as: Some(CardKind::Shan),
                    cancelled: false,
                }));
                machine.stage = HostStage::DrawSettle;
                Flow::Yield
            } else {
                wait_ask_fix(machine, ctx, HostStage::TargetChoice, 1029)
            }
        }
        HostStage::JiefenTarget => {
            // Defensive `sum == threshold` after `start_draw` settles: emit `CardUsed(Shan)` then yield before park resume!
            ctx.emit(EventData::CardUsed(CardUseData {
                card: None,
                player: owner,
                targets: Vec::new(),
                nature: DamageNature::Normal,
                use_as: Some(CardKind::Shan),
                cancelled: false,
            }));
            machine.stage = HostStage::DrawSettle;
            Flow::Yield
        }
        HostStage::TargetChoice => {
            let Some(ChoiceOption::Player(t)) = ctx.answer().first().copied() else {
                return Flow::Done;
            };
            if !is_sha_legal_play_target(ctx.core, ctx.catalog, owner, t) {
                return Flow::Done;
            }
            set_turn_flag(ctx.core, owner, "dddshilie_used_turn", "dddshilie_stamp", 1);
            for &c in &machine.cards {
                set_flag(ctx.core, owner, &format!("dddshilie_shown_card:{c}"), 1);
            }
            ctx.log(LogEvent::CardsRevealed {
                cards: machine.cards.clone(),
                to: ctx.core.alive_players().collect(),
            });
            consume_sha_play_phase(ctx.core, ctx.catalog, owner);
            ctx.log(LogEvent::SkillTriggered {
                skill: "dddshilie",
                owner,
                presentation: None,
            });
            machine.target = Some(t);
            if machine.targets.first().copied() == Some(1) {
                ctx.start_draw(owner, machine.cards.len() as u32);
                machine.stage = HostStage::DrawSettle;
                Flow::Yield
            } else {
                ctx.emit(EventData::CardUsed(CardUseData {
                    card: None,
                    player: owner,
                    targets: vec![t],
                    nature: DamageNature::Normal,
                    use_as: Some(CardKind::Sha),
                    cancelled: false,
                }));
                Flow::Done
            }
        }
        HostStage::DrawSettle => {
            if is_defensive {
                finish_respond_park(ctx, owner, true)
            } else {
                if let Some(t) = machine.target {
                    if is_sha_legal_play_target(ctx.core, ctx.catalog, owner, t) {
                        ctx.emit(EventData::CardUsed(CardUseData {
                            card: None,
                            player: owner,
                            targets: vec![t],
                            nature: DamageNature::Normal,
                            use_as: Some(CardKind::Sha),
                            cancelled: false,
                        }));
                    }
                }
                Flow::Done
            }
        }
        _ => Flow::Done,
    }
}

fn bingjian_enter_adjust(
    machine: &mut HostMachine,
    ctx: &mut Ctx<'_>,
    owner: PlayerId,
    mode_flag: u8,
    sha_target: PlayerId,
) -> Flow {
    let hlen = ctx.core.player(owner).hand.len();
    if hlen < 2 {
        let diff = 2 - hlen;
        let prev = turn_flag(ctx.core, owner, "dddbingjian_used_count", "dddbingjian_stamp");
        set_turn_flag(ctx.core, owner, "dddbingjian_used_count", "dddbingjian_stamp", prev + 1);
        machine.targets = vec![diff as u8, 1, mode_flag, sha_target];
        machine.card = None;
        ctx.start_draw(owner, diff as u32);
        machine.stage = HostStage::DrawSettle;
        Flow::Yield
    } else if hlen > 2 {
        let diff = hlen - 2;
        machine.targets = vec![diff as u8, 0, mode_flag, sha_target];
        machine.card = None;
        wait_ask_fix(machine, ctx, HostStage::SelectCard, 1030)
    } else if mode_flag >= 10 {
        finish_respond_park(ctx, owner, false)
    } else {
        Flow::Done
    }
}

fn bingjian_emit_and_finish(machine: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = machine.owner;
    let mode_flag = machine.targets.get(2).copied().unwrap_or(0);
    if mode_flag < 10 {
        let sha_t = machine.targets.get(3).copied().unwrap_or(0);
        if is_sha_legal_play_target(ctx.core, ctx.catalog, owner, sha_t) {
            consume_sha_play_phase(ctx.core, ctx.catalog, owner);
            ctx.log(LogEvent::SkillTriggered {
                skill: "dddbingjian",
                owner,
                presentation: None,
            });
            ctx.emit(EventData::CardUsed(CardUseData {
                card: None,
                player: owner,
                targets: vec![sha_t],
                nature: DamageNature::Normal,
                use_as: Some(CardKind::Sha),
                cancelled: false,
            }));
        }
        Flow::Done
    } else if mode_flag == 10 {
        ctx.log(LogEvent::SkillTriggered {
            skill: "dddbingjian",
            owner,
            presentation: None,
        });
        ctx.emit(EventData::CardUsed(CardUseData {
            card: None,
            player: owner,
            targets: Vec::new(),
            nature: DamageNature::Normal,
            use_as: Some(CardKind::Shan),
            cancelled: false,
        }));
        machine.card = Some(21);
        machine.stage = HostStage::DrawSettle;
        Flow::Yield
    } else {
        let resp_kind = if mode_flag == 12 { CardKind::Shan } else { CardKind::Sha };
        ctx.log(LogEvent::SkillTriggered {
            skill: "dddbingjian",
            owner,
            presentation: None,
        });
        maybe_trigger_fushi(ctx, owner, resp_kind);
        machine.stage = HostStage::JiefenTarget;
        Flow::Yield
    }
}

fn step_bingjian(machine: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = machine.owner;
    match machine.stage {
        HostStage::Act => {
            let mode_flag = machine.card.unwrap_or(0) as u8;
            if mode_flag < 10 {
                wait_ask_fix(machine, ctx, HostStage::SelectNature, 1033)
            } else {
                bingjian_enter_adjust(machine, ctx, owner, mode_flag, 0)
            }
        }
        HostStage::SelectNature => {
            let Some(ChoiceOption::Player(t)) = ctx.answer().first().copied() else {
                return Flow::Done;
            };
            if !is_sha_legal_play_target(ctx.core, ctx.catalog, owner, t) {
                return Flow::Done;
            }
            bingjian_enter_adjust(machine, ctx, owner, 0, t)
        }
        HostStage::SelectCard => {
            let mode_flag = machine.targets.get(2).copied().unwrap_or(0);
            let picks: Vec<CardId> = ctx
                .answer()
                .iter()
                .filter_map(|o| match o {
                    ChoiceOption::Card(c) => Some(*c),
                    _ => None,
                })
                .filter(|c| ctx.core.player(owner).hand.contains(c))
                .collect();
            let diff = machine.targets.first().copied().unwrap_or(0) as usize;
            if picks.len() != diff || picks.is_empty() {
                return if mode_flag >= 10 {
                    finish_respond_park(ctx, owner, false)
                } else {
                    Flow::Done
                };
            }
            let prev = turn_flag(ctx.core, owner, "dddbingjian_used_count", "dddbingjian_stamp");
            set_turn_flag(ctx.core, owner, "dddbingjian_used_count", "dddbingjian_stamp", prev + 1);
            u01_discard_batch(ctx, owner, &picks);
            machine.card = None;
            machine.stage = HostStage::DrawSettle;
            Flow::Yield
        }
        HostStage::DrawSettle => {
            if machine.card == Some(21) {
                return finish_respond_park(ctx, owner, true);
            }
            if machine.card == Some(20) {
                return bingjian_emit_and_finish(machine, ctx);
            }
            wait_ask_fix(machine, ctx, HostStage::TargetChoice, 1031)
        }
        HostStage::TargetChoice => {
            let diff = machine.targets.first().copied().unwrap_or(0);
            let is_draw = machine.targets.get(1).copied() == Some(1);
            if let Some(ChoiceOption::Player(t)) = ctx.answer().first().copied() {
                if ctx.core.player(t).alive {
                    if is_draw {
                        machine.card = Some(20);
                        ctx.start_draw(t, u32::from(diff));
                        machine.stage = HostStage::DrawSettle;
                        return Flow::Yield;
                    } else if !he_cards(ctx.core, t).is_empty() && diff > 0 {
                        machine.target = Some(t);
                        machine.card = Some(u32::from(diff));
                        let mut shuffled = ctx.core.player(t).hand.clone();
                        if shuffled.len() > 1 {
                            ctx.core.rng.shuffle(&mut shuffled);
                        }
                        machine.cards = shuffled;
                        return wait_ask_fix(machine, ctx, HostStage::CardsAsk, 1032);
                    }
                }
            }
            bingjian_emit_and_finish(machine, ctx)
        }
        HostStage::CardsAsk => {
            if let Some(t) = machine.target {
                let mut to_disc = Vec::new();
                for opt in ctx.answer() {
                    match *opt {
                        ChoiceOption::HiddenCard(slot) => {
                            if let Some(&c) = machine.cards.get(slot as usize) {
                                if ctx.core.player(t).hand.contains(&c) {
                                    to_disc.push(c);
                                }
                            }
                        }
                        ChoiceOption::Card(c) => {
                            if ctx.core.player(t).equips.contains(&c) {
                                to_disc.push(c);
                            }
                        }
                        _ => {}
                    }
                }
                u01_discard_batch(ctx, t, &to_disc);
            }
            machine.card = Some(20);
            machine.stage = HostStage::DrawSettle;
            Flow::Yield
        }
        HostStage::JiefenTarget => finish_respond_park(ctx, owner, true),
        _ => Flow::Done,
    }
}

fn step_tongyu(machine: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = machine.owner;
    if machine.card == Some(2) {
        return match machine.stage {
            HostStage::Act => wait_ask_fix(machine, ctx, HostStage::CardsAsk, 1037),
            HostStage::CardsAsk => {
                let pile = ctx
                    .core
                    .vm_aux
                    .piles
                    .remove(&(owner, "dddtongyu_wugu_rem".to_string()))
                    .unwrap_or_default();
                if let Some(ChoiceOption::Player(t)) = ctx.answer().first().copied() {
                    if ctx.core.player(t).alive && !pile.is_empty() {
                        ctx.core.player_mut(t).hand.extend(pile.iter().copied());
                        ctx.log(LogEvent::CardsGiven {
                            from: owner,
                            to: t,
                            cards: pile,
                        });
                        return Flow::Done;
                    }
                }
                for c in pile {
                    ctx.to_discard(c);
                }
                Flow::Done
            }
            _ => Flow::Done,
        };
    }
    if machine.card == Some(1) {
        return match machine.stage {
            HostStage::Act => {
                let fid = ctx.current_frame_id();
                ctx.core
                    .vm_aux
                    .flags
                    .remove(&(owner, "dddtongyu_pending_go".to_string()));
                let extra = ctx
                    .core
                    .vm_aux
                    .flags
                    .remove(&(owner, "dddtongyu_pending_extra".to_string()))
                    .unwrap_or(0);
                set_flag(ctx.core, owner, &format!("dddtongyu_wugu_extra:{fid}"), extra);
                set_flag(ctx.core, owner, &format!("dddtongyu_wugu_rem_owner:{fid}"), 1);
                machine.cards = vec![fid];
                machine.targets = ctx.core.seat_order_from(ctx.core.turn.current);
                tongyu_next_target(machine, ctx)
            }
            HostStage::TargetChoice => {
                let fid = machine.cards.first().copied().unwrap_or(0);
                if let (Some(t), Some(ChoiceOption::Tag(tag))) =
                    (machine.target, ctx.answer().first().copied())
                {
                    if tag == 0 {
                        set_flag(ctx.core, t, "dddtongyu_hand_locked_stamp", l28_stamp(ctx.core));
                    } else {
                        set_flag(ctx.core, owner, &format!("dddtongyu_wugu_exclude:{fid}:{t}"), 1);
                        set_flag(ctx.core, owner, &format!("dddtongyu_excluded:{fid}:{t}"), 1);
                    }
                }
                tongyu_next_target(machine, ctx)
            }
            HostStage::SelectNature => {
                let ccw = matches!(ctx.answer().first().copied(), Some(ChoiceOption::Tag(1)));
                let fid = machine.cards.first().copied().unwrap_or(0);
                if ccw {
                    set_flag(ctx.core, owner, &format!("dddtongyu_wugu_ccw:{fid}"), 1);
                    if let Some(frame) = ctx.res.frame_mut(fid) {
                        let targets = &mut frame.card_use_mut().targets;
                        if targets.len() > 2 {
                            targets[1..].reverse();
                        }
                    }
                }
                Flow::Done
            }
            _ => Flow::Done,
        };
    }
    match machine.stage {
        HostStage::Act => wait_ask_fix(machine, ctx, HostStage::SelectCard, 1034),
        HostStage::SelectCard => {
            let picks: Vec<CardId> = ctx
                .answer()
                .iter()
                .filter_map(|o| match o {
                    ChoiceOption::Card(c) => Some(*c),
                    _ => None,
                })
                .filter(|c| ctx.core.player(owner).hand.contains(c))
                .collect();
            if picks.is_empty()
                || effective_distinct_suits(ctx.core, ctx.catalog, owner, &picks) != picks.len()
            {
                return Flow::Done;
            }
            mark_scope4_used(ctx.core, owner, "dddtongyu");
            set_flag(ctx.core, owner, "dddtongyu_paid_count", picks.len() as u32);
            for (idx, &c) in picks.iter().enumerate() {
                ctx.take_hand_card(owner, c);
                ctx.core.table.push(c);
                set_flag(ctx.core, owner, &format!("dddtongyu_paid:{idx}"), c);
            }
            let lost_n = picks.len();
            machine.cards = picks.clone();
            ctx.emit(EventData::CardsLost {
                player: owner,
                cards: picks,
                origins: vec![LostOrigin::Hand; lost_n],
            });
            machine.stage = HostStage::DrawSettle;
            Flow::Yield
        }
        HostStage::DrawSettle => {
            if !ctx.core.player(owner).alive {
                return Flow::Done;
            }
            let extra = machine.cards.len() as u32;
            set_flag(ctx.core, owner, "dddtongyu_pending_extra", extra);
            set_flag(ctx.core, owner, "dddtongyu_pending_go", 1);
            let targets: Vec<PlayerId> = ctx.core.seat_order_from(owner);
            ctx.log(LogEvent::SkillTriggered {
                skill: "dddtongyu",
                owner,
                presentation: None,
            });
            ctx.emit(EventData::CardUsed(CardUseData {
                card: None,
                player: owner,
                targets,
                nature: DamageNature::Normal,
                use_as: Some(CardKind::Wugu),
                cancelled: false,
            }));
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn tongyu_next_target(machine: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    while !machine.targets.is_empty() {
        let t = machine.targets.remove(0);
        if ctx.core.player(t).alive {
            machine.target = Some(t);
            return wait_ask_fix(machine, ctx, HostStage::TargetChoice, 1036);
        }
    }
    wait_ask_fix(machine, ctx, HostStage::SelectNature, 1035)
}
