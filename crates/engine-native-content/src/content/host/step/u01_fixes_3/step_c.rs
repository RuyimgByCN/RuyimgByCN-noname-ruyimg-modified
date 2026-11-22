use super::*;

pub(super) fn step_u01_fix3_c(machine: &mut HostMachine, ctx: &mut Ctx<'_>) -> Option<Flow> {
    let f = match machine.def_key.as_str() {
        "dddxiaheng" => step_dddxiaheng(machine, ctx),
        "dddfengzheng" => step_dddfengzheng(machine, ctx),
        "dddfengzheng_global" => step_dddfengzheng_global(machine, ctx),
        "dddyulv" => step_dddyulv(machine, ctx),
        "dddyulv_old" => step_dddyulv_old(machine, ctx),
        "dddfenye" => step_dddfenye(machine, ctx),
        "dddshichao" => step_dddshichao(machine, ctx),
        _ => return None,
    };
    Some(f)
}

fn step_dddxiaheng(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.stage {
        HostStage::Act => u01_wait_ask(m, ctx, HostStage::TargetChoice, 3001),
        HostStage::TargetChoice => {
            u01_clear_ask(ctx, owner);
            let Some(ChoiceOption::Player(t1)) = ctx.answer().first().copied() else {
                return Flow::Done;
            };
            m.target = Some(t1);
            let he_len = he_cards(ctx.core, t1).len();
            if he_len == 0 {
                u01_wait_ask(m, ctx, HostStage::SelectNature, 3003)
            } else if he_len <= 2 {
                let drops = he_cards(ctx.core, t1);
                u01_discard_batch(ctx, t1, &drops);
                u01_wait_ask(m, ctx, HostStage::SelectNature, 3003)
            } else {
                u01_wait_ask(m, ctx, HostStage::SelectCard, 3002)
            }
        }
        HostStage::SelectCard => {
            u01_clear_ask(ctx, owner);
            let Some(t1) = m.target else {
                return Flow::Done;
            };
            let drops: Vec<CardId> = ctx
                .answer()
                .iter()
                .filter_map(|o| match o {
                    ChoiceOption::Card(c) => Some(*c),
                    _ => None,
                })
                .collect();
            if !drops.is_empty() {
                u01_discard_batch(ctx, t1, &drops);
            }
            u01_wait_ask(m, ctx, HostStage::SelectNature, 3003)
        }
        HostStage::SelectNature => {
            u01_clear_ask(ctx, owner);
            let Some(ChoiceOption::Player(t2)) = ctx.answer().first().copied() else {
                return Flow::Done;
            };
            m.targets = vec![t2];
            ctx.emit(EventData::Damage(damage_data_with(
                Some(owner),
                t2,
                1,
                DamageNature::Normal,
                None,
            )));
            m.stage = HostStage::ShuchuiTarget;
            Flow::Yield
        }
        HostStage::ShuchuiTarget => {
            let Some(t1) = m.target else {
                return Flow::Done;
            };
            let Some(&t2) = m.targets.first() else {
                return Flow::Done;
            };
            let already_del = flag(ctx.core, owner, "dddxiaheng_del") != 0;
            if !already_del && t1 != owner && t2 != owner && ctx.core.player(owner).alive {
                ctx.lose_max_hp(owner, 1);
            }
            ctx.log(LogEvent::SkillTriggered {
                skill: "dddxiaheng",
                owner,
                presentation: None,
            });
            if t1 == t2 {
                set_flag(ctx.core, owner, "dddxiaheng_same_target", 1);
                if let Some(def) = ctx.catalog.skill_by_name("dddxiaheng") {
                    ctx.remove_skill(owner, def);
                }
            } else {
                set_flag(ctx.core, owner, "dddxiaheng_same_target", 0);
                let prev_mask = flag(ctx.core, owner, "dddxiaheng_damaged_mask");
                let new_mask = prev_mask | (1u32 << t2);
                set_flag(ctx.core, owner, "dddxiaheng_damaged_mask", new_mask);
                if new_mask.count_ones() >= 3 {
                    set_flag(ctx.core, owner, "dddxiaheng_del", 1);
                }
            }
            if ctx.core.player(owner).max_hp == 0 && ctx.core.player(owner).alive {
                ctx.emit(EventData::Death {
                    player: owner,
                    killer: None,
                });
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn step_dddfengzheng_global(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let actor = m.owner;
    match m.stage {
        HostStage::Act => u01_wait_ask(m, ctx, HostStage::SelectCard, 3004),
        HostStage::SelectCard => {
            u01_clear_ask(ctx, actor);
            let picks: Vec<CardId> = ctx
                .answer()
                .iter()
                .filter_map(|o| match o {
                    ChoiceOption::Card(c) => Some(*c),
                    _ => None,
                })
                .collect();
            if picks.is_empty() {
                return Flow::Done;
            }
            let sum: u32 = picks
                .iter()
                .map(|&c| u32::from(ctx.catalog.cards[c as usize].rank))
                .sum();
            if sum != 13 {
                return Flow::Done;
            }
            let tok = play_phase_token(ctx.core);
            set_flag(ctx.core, actor, "dddfengzheng_global_used_tok", tok);
            let r = ctx.core.turn.round.max(1);
            let prev_mask = flag(ctx.core, 0, &format!("dddfengzheng_round:{r}_mask"));
            set_flag(
                ctx.core,
                0,
                &format!("dddfengzheng_round:{r}_mask"),
                prev_mask | (1u32 << actor),
            );
            for &c in &picks[1..] {
                ctx.take_hand_card(actor, c);
                ctx.to_discard(c);
            }
            if picks.len() > 1 {
                ctx.note_cards_lost(actor, &picks[1..]);
            }
            ctx.emit(EventData::CardUsed(CardUseData {
                card: Some(picks[0]),
                player: actor,
                targets: vec![actor],
                nature: DamageNature::Normal,
                use_as: Some(CardKind::Wuzhong),
                cancelled: false,
            }));
            ctx.log(LogEvent::SkillTriggered {
                skill: "dddfengzheng_global",
                owner: actor,
                presentation: None,
            });
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn step_dddfengzheng(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.stage {
        HostStage::Act => {
            let r = ctx.core.turn.round.max(1);
            set_flag(ctx.core, owner, "dddfengzheng_last_round", r);
            if r <= 1 {
                return Flow::Done;
            }
            let prev_r = r - 1;
            let mask = flag(ctx.core, 0, &format!("dddfengzheng_round:{prev_r}_mask"));
            let num = ctx
                .core
                .alive_players()
                .filter(|&p| (mask & (1u32 << p)) != 0)
                .count() as u32;
            if num == 0 {
                return Flow::Done;
            }
            for p in 0..ctx.core.players.len() as PlayerId {
                ctx.core
                    .vm_aux
                    .flags
                    .remove(&(owner, format!("dddfengzheng_dist_cnt:{p}")));
            }
            ctx.core
                .vm_aux
                .flags
                .remove(&(owner, "dddfengzheng_lose_flag".to_string()));
            m.card = Some(num);
            m.cards.clear();
            u01_wait_ask(m, ctx, HostStage::ShenmiguoRestage, 3023)
        }
        HostStage::ShenmiguoRestage => {
            u01_clear_ask(ctx, owner);
            if !matches!(ctx.answer().first(), Some(ChoiceOption::Confirm)) {
                return Flow::Done;
            }
            let num = m.card.unwrap_or(0);
            m.cards.clear();
            for _ in 0..num {
                if ctx.core.deck.is_empty() && !ctx.core.discard.is_empty() {
                    let mut recycle = std::mem::take(&mut ctx.core.discard);
                    ctx.core.rng.shuffle(&mut recycle);
                    ctx.core.deck = recycle;
                }
                let Some(c) = ctx.core.deck.pop() else {
                    break;
                };
                ctx.core.player_mut(owner).off_court.push(c);
                m.cards.push(c);
            }
            if m.cards.is_empty() {
                return Flow::Done;
            }
            ctx.log(LogEvent::SkillTriggered {
                skill: "dddfengzheng",
                owner,
                presentation: None,
            });
            u01_wait_ask(m, ctx, HostStage::TargetChoice, 3005)
        }
        HostStage::TargetChoice => {
            u01_clear_ask(ctx, owner);
            let Some(ChoiceOption::Player(tgt)) = ctx.answer().first().copied() else {
                return finish_dddfengzheng_with_leftovers(m, ctx);
            };
            m.target = Some(tgt);
            if m.cards.is_empty() {
                return finish_dddfengzheng(ctx, owner);
            }
            u01_wait_ask(m, ctx, HostStage::SelectCard, 3006)
        }
        HostStage::SelectCard => {
            u01_clear_ask(ctx, owner);
            let Some(tgt) = m.target else {
                return finish_dddfengzheng_with_leftovers(m, ctx);
            };
            let chosen: Vec<CardId> = ctx
                .answer()
                .iter()
                .filter_map(|o| match o {
                    ChoiceOption::Card(c)
                        if m.cards.contains(c) && ctx.core.player(owner).off_court.contains(c) =>
                    {
                        Some(*c)
                    }
                    _ => None,
                })
                .collect();
            if !chosen.is_empty() {
                ctx.core
                    .player_mut(owner)
                    .off_court
                    .retain(|x| !chosen.contains(x));
                m.cards.retain(|x| !chosen.contains(x));
                for &c in &chosen {
                    ctx.core.player_mut(tgt).hand.push(c);
                }
                ctx.log(LogEvent::CardsGiven {
                    from: owner,
                    to: tgt,
                    cards: chosen.clone(),
                });
                ctx.emit(EventData::CardsDrawn {
                    player: tgt,
                    cards: chosen.clone(),
                });
                let prev_cnt = flag(ctx.core, owner, &format!("dddfengzheng_dist_cnt:{tgt}"));
                let next_cnt = prev_cnt + chosen.len() as u32;
                set_flag(
                    ctx.core,
                    owner,
                    &format!("dddfengzheng_dist_cnt:{tgt}"),
                    next_cnt,
                );
                if next_cnt > 2 {
                    set_flag(ctx.core, owner, "dddfengzheng_lose_flag", 1);
                }
            }
            if m.cards.is_empty() || chosen.is_empty() {
                finish_dddfengzheng_with_leftovers(m, ctx)
            } else {
                u01_wait_ask(m, ctx, HostStage::TargetChoice, 3005)
            }
        }
        _ => Flow::Done,
    }
}

fn finish_dddfengzheng_with_leftovers(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    if !m.cards.is_empty() {
        let leftovers = std::mem::take(&mut m.cards);
        ctx.core
            .player_mut(owner)
            .off_court
            .retain(|x| !leftovers.contains(x));
        for c in leftovers {
            ctx.to_discard(c);
        }
    }
    finish_dddfengzheng(ctx, owner)
}

fn finish_dddfengzheng(ctx: &mut Ctx<'_>, owner: PlayerId) -> Flow {
    if ctx
        .core
        .vm_aux
        .flags
        .remove(&(owner, "dddfengzheng_lose_flag".to_string()))
        .is_some()
    {
        if let Some(def) = ctx.catalog.skill_by_name("dddfengzheng") {
            ctx.remove_skill(owner, def);
        }
    }
    Flow::Done
}

fn step_dddyulv(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    // `m.card == None`: Prepare initialization; `Some(1)`: discard match trigger.
    if m.card.is_none() {
        match m.stage {
            HostStage::Act => {
                set_flag(ctx.core, owner, "dddyulv_init", 1);
                ctx.log(LogEvent::SkillTriggered {
                    skill: "dddyulv",
                    owner,
                    presentation: None,
                });
                ctx.start_draw(owner, 1);
                m.stage = HostStage::DrawSettle;
                Flow::Yield
            }
            HostStage::DrawSettle => {
                if ctx.core.player(owner).hand.is_empty() {
                    Flow::Done
                } else {
                    u01_wait_ask(m, ctx, HostStage::SelectCard, 3007)
                }
            }
            HostStage::SelectCard => {
                u01_clear_ask(ctx, owner);
                if let Some(ChoiceOption::Card(c)) = ctx.answer().first().copied() {
                    u01_remove_owned_card_quiet(ctx, owner, c);
                    ctx.note_cards_lost(owner, &[c]);
                    ctx.core
                        .vm_aux
                        .piles
                        .entry((owner, "dddyulv".to_string()))
                        .or_default()
                        .push(c);
                }
                Flow::Done
            }
            _ => Flow::Done,
        }
    } else {
        match m.stage {
            HostStage::Act => {
                let Some(cur) = m.target else {
                    return Flow::Done;
                };
                if !ctx.core.player(cur).alive {
                    return Flow::Done;
                }
                ctx.log(LogEvent::SkillTriggered {
                    skill: "dddyulv",
                    owner,
                    presentation: None,
                });
                u01_wait_ask(m, ctx, HostStage::SelectNature, 3008)
            }
            HostStage::SelectNature => {
                u01_clear_ask(ctx, owner);
                let Some(cur) = m.target else {
                    return Flow::Done;
                };
                match ctx.answer().first().copied() {
                    Some(ChoiceOption::Tag(1)) if !he_cards(ctx.core, cur).is_empty() => {
                        u01_wait_ask(m, ctx, HostStage::SelectCard, 3009)
                    }
                    _ => {
                        ctx.start_draw(cur, 1);
                        m.stage = HostStage::DrawSettle;
                        Flow::Yield
                    }
                }
            }
            HostStage::SelectCard => {
                u01_clear_ask(ctx, owner);
                let Some(cur) = m.target else {
                    return Flow::Done;
                };
                if let Some(ChoiceOption::Card(c)) = ctx.answer().first().copied() {
                    u01_discard_batch(ctx, cur, &[c]);
                }
                dddyulv_maybe_second_swap(m, ctx)
            }
            HostStage::DrawSettle => dddyulv_maybe_second_swap(m, ctx),
            HostStage::HuanrenPick => {
                if ctx.core.player(owner).hand.is_empty() {
                    Flow::Done
                } else {
                    u01_wait_ask(m, ctx, HostStage::CompareTargetPick, 3010)
                }
            }
            HostStage::CompareTargetPick => {
                u01_clear_ask(ctx, owner);
                if let Some(ChoiceOption::Card(c)) = ctx.answer().first().copied() {
                    if ctx.core.player(owner).hand.contains(&c) {
                        let old = ctx
                            .core
                            .vm_aux
                            .piles
                            .get_mut(&(owner, "dddyulv".to_string()))
                            .and_then(Vec::pop);
                        if let Some(old) = old {
                            ctx.take_hand_card(owner, c);
                            ctx.core
                                .vm_aux
                                .piles
                                .entry((owner, "dddyulv".to_string()))
                                .or_default()
                                .push(c);
                            ctx.core.player_mut(owner).hand.push(old);
                        }
                    }
                }
                Flow::Done
            }
            _ => Flow::Done,
        }
    }
}

fn dddyulv_maybe_second_swap(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    let cnt = flag(ctx.core, owner, "dddyulv_turn_cnt") + 1;
    set_flag(ctx.core, owner, "dddyulv_turn_cnt", cnt);
    if cnt != 2 || !ctx.core.player(owner).alive {
        return Flow::Done;
    }
    if ctx.core.player(owner).hand.is_empty() {
        ctx.start_draw(owner, 1);
        m.stage = HostStage::HuanrenPick;
        Flow::Yield
    } else {
        u01_wait_ask(m, ctx, HostStage::CompareTargetPick, 3010)
    }
}

fn step_dddyulv_old(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    if m.card.is_none() {
        match m.stage {
            HostStage::Act => u01_wait_ask(m, ctx, HostStage::SelectNature, 3011),
            HostStage::SelectNature => {
                u01_clear_ask(ctx, owner);
                if let Some(ChoiceOption::Tag(r)) = ctx.answer().first().copied() {
                    set_flag(ctx.core, owner, "dddyulv_old_rank", u32::from(r));
                    ctx.log(LogEvent::SkillTriggered {
                        skill: "dddyulv_old",
                        owner,
                        presentation: None,
                    });
                }
                Flow::Done
            }
            _ => Flow::Done,
        }
    } else {
        match m.stage {
            HostStage::Act => {
                let Some(cur) = m.target else {
                    return Flow::Done;
                };
                if !ctx.core.player(cur).alive {
                    return Flow::Done;
                }
                ctx.log(LogEvent::SkillTriggered {
                    skill: "dddyulv_old",
                    owner,
                    presentation: None,
                });
                u01_wait_ask(m, ctx, HostStage::SelectNature, 3012)
            }
            HostStage::SelectNature => {
                u01_clear_ask(ctx, owner);
                let Some(cur) = m.target else {
                    return Flow::Done;
                };
                let opt = match ctx.answer().first().copied() {
                    Some(ChoiceOption::Tag(1)) if !he_cards(ctx.core, cur).is_empty() => 1u32,
                    _ => 0u32,
                };
                m.card = Some(1 + opt); // 1 => opt 0, 2 => opt 1
                if opt == 1 {
                    u01_wait_ask(m, ctx, HostStage::SelectCard, 3013)
                } else {
                    ctx.start_draw(cur, 1);
                    m.stage = HostStage::DrawSettle;
                    Flow::Yield
                }
            }
            HostStage::SelectCard => {
                u01_clear_ask(ctx, owner);
                let Some(cur) = m.target else {
                    return Flow::Done;
                };
                if let Some(ChoiceOption::Card(c)) = ctx.answer().first().copied() {
                    u01_discard_batch(ctx, cur, &[c]);
                }
                dddyulv_old_after_choice(m, ctx)
            }
            HostStage::DrawSettle => dddyulv_old_after_choice(m, ctx),
            HostStage::CompareTargetPick => {
                u01_clear_ask(ctx, owner);
                if let Some(ChoiceOption::Tag(r)) = ctx.answer().first().copied() {
                    set_flag(ctx.core, owner, "dddyulv_old_rank", u32::from(r));
                }
                if let Some(cur) = m.target {
                    if ctx.core.turn.current == cur {
                        if ctx.res.stack.is_empty() {
                            ctx.emit(EventData::TurnEnded { player: cur });
                        } else {
                            set_flag(ctx.core, cur, "u01_abort_turn", 1);
                            ctx.core.turn.skip_play_phase = true;
                        }
                    }
                }
                Flow::Done
            }
            _ => Flow::Done,
        }
    }
}

fn dddyulv_old_after_choice(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    let Some(cur) = m.target else {
        return Flow::Done;
    };
    let chosen_code = m.card.unwrap_or(1); // 1 or 2
    let key = format!("dddyulv_old_prev_opt:{cur}");
    let prev = flag(ctx.core, owner, &key);
    if prev == chosen_code && ctx.core.player(owner).alive {
        // Same option consecutively: re-declare 1..=13 and end cur's current turn!
        u01_wait_ask(m, ctx, HostStage::CompareTargetPick, 3011)
    } else {
        set_flag(ctx.core, owner, &key, chosen_code);
        Flow::Done
    }
}

fn step_dddfenye(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.stage {
        HostStage::Act => {
            for p in 0..ctx.core.players.len() as PlayerId {
                ctx.core
                    .vm_aux
                    .flags
                    .remove(&(owner, format!("u01_fenye_pick:{p}")));
                ctx.core
                    .vm_aux
                    .flags
                    .remove(&(owner, format!("u01_fenye_side:{p}")));
            }
            ctx.core
                .vm_aux
                .flags
                .remove(&(owner, "dddfenye_losers_mask".to_string()));
            u01_wait_ask(m, ctx, HostStage::TargetChoice, 3014)
        }
        HostStage::TargetChoice => {
            u01_clear_ask(ctx, owner);
            let Some(ChoiceOption::Player(t)) = ctx.answer().first().copied() else {
                return Flow::Done;
            };
            m.target = Some(t);
            u01_wait_ask(m, ctx, HostStage::SelectCard, 3015)
        }
        HostStage::SelectCard => {
            u01_clear_ask(ctx, owner);
            let Some(ChoiceOption::Card(c)) = ctx.answer().first().copied() else {
                return Flow::Done;
            };
            set_flag(ctx.core, owner, &format!("u01_fenye_pick:{owner}"), c);
            set_flag(ctx.core, owner, &format!("u01_fenye_side:{owner}"), 0);
            set_flag(ctx.core, owner, "dddfenye_used_tok", play_phase_token(ctx.core));
            ctx.log(LogEvent::SkillTriggered {
                skill: "dddfenye",
                owner,
                presentation: None,
            });
            u01_wait_ask(m, ctx, HostStage::JiefenTargetDiscard, 3016)
        }
        HostStage::JiefenTargetDiscard => {
            u01_clear_ask(ctx, owner);
            let Some(t) = m.target else {
                return Flow::Done;
            };
            let Some(ChoiceOption::Card(c)) = ctx.answer().first().copied() else {
                return Flow::Done;
            };
            set_flag(ctx.core, owner, &format!("u01_fenye_pick:{t}"), c);
            set_flag(ctx.core, owner, &format!("u01_fenye_side:{t}"), 1);
            m.targets = ctx
                .core
                .seat_order_from(owner)
                .into_iter()
                .filter(|&p| p != owner && p != t && !ctx.core.player(p).hand.is_empty())
                .collect();
            if m.targets.is_empty() {
                dddfenye_resolve_groups(m, ctx)
            } else {
                u01_wait_ask(m, ctx, HostStage::ComparePlayerPick, 3017)
            }
        }
        HostStage::ComparePlayerPick => {
            u01_clear_ask(ctx, owner);
            let Some(&bystander) = m.targets.first() else {
                return dddfenye_resolve_groups(m, ctx);
            };
            if let Some(ChoiceOption::Card(c)) = ctx.answer().first().copied() {
                set_flag(ctx.core, owner, &format!("u01_fenye_pick:{bystander}"), c);
                return u01_wait_ask(m, ctx, HostStage::SelectNature, 3018);
            }
            m.targets.remove(0);
            if m.targets.is_empty() {
                dddfenye_resolve_groups(m, ctx)
            } else {
                u01_wait_ask(m, ctx, HostStage::ComparePlayerPick, 3017)
            }
        }
        HostStage::SelectNature => {
            u01_clear_ask(ctx, owner);
            let Some(&bystander) = m.targets.first() else {
                return dddfenye_resolve_groups(m, ctx);
            };
            let t = m.target.unwrap_or(owner);
            match ctx.answer().first().copied() {
                Some(ChoiceOption::Player(side_player)) if side_player == owner || side_player == t => {
                    let side = u32::from(side_player == t);
                    set_flag(ctx.core, owner, &format!("u01_fenye_side:{bystander}"), side);
                }
                _ => {
                    ctx.core
                        .vm_aux
                        .flags
                        .remove(&(owner, format!("u01_fenye_pick:{bystander}")));
                }
            }
            m.targets.remove(0);
            if m.targets.is_empty() {
                dddfenye_resolve_groups(m, ctx)
            } else {
                u01_wait_ask(m, ctx, HostStage::ComparePlayerPick, 3017)
            }
        }
        HostStage::ShuchuiTargetLoop | HostStage::JiefenOwnerDiscard => {
            u01_clear_ask(ctx, owner);
            if m.stage == HostStage::ShuchuiTargetLoop {
                let Some(winner) = m.targets.first().copied() else {
                    return Flow::Done;
                };
                if let Some(ChoiceOption::Player(loser)) = ctx.answer().first().copied() {
                    let lmask = flag(ctx.core, owner, "dddfenye_losers_mask");
                    set_flag(ctx.core, owner, "dddfenye_losers_mask", lmask & !(1u32 << loser));
                    m.targets.remove(0);
                    ctx.emit(EventData::CardUsed(CardUseData {
                        card: None,
                        player: winner,
                        targets: vec![loser],
                        nature: DamageNature::Normal,
                        use_as: Some(CardKind::Sha),
                        cancelled: false,
                    }));
                    m.stage = HostStage::JiefenOwnerDiscard;
                    return Flow::Yield;
                }
                m.targets.remove(0);
            }
            dddfenye_next_winner(m, ctx)
        }
        _ => Flow::Done,
    }
}

fn dddfenye_resolve_groups(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    let seats = ctx.core.players.len() as PlayerId;
    let mut sum0: u64 = 0;
    let mut cnt0: u64 = 0;
    let mut sum1: u64 = 0;
    let mut cnt1: u64 = 0;
    let mut g0 = Vec::new();
    let mut g1 = Vec::new();
    for p in ctx.core.seat_order_from(owner) {
        let pick = flag(ctx.core, owner, &format!("u01_fenye_pick:{p}"));
        if pick == 0 {
            continue;
        }
        let side = flag(ctx.core, owner, &format!("u01_fenye_side:{p}"));
        let rank = u64::from(ctx.catalog.cards[pick as usize].rank);
        u01_discard_batch(ctx, p, &[pick]);
        if side == 0 {
            sum0 += rank;
            cnt0 += 1;
            g0.push(p);
        } else {
            sum1 += rank;
            cnt1 += 1;
            g1.push(p);
        }
    }
    for p in 0..seats {
        ctx.core
            .vm_aux
            .flags
            .remove(&(owner, format!("u01_fenye_pick:{p}")));
        ctx.core
            .vm_aux
            .flags
            .remove(&(owner, format!("u01_fenye_side:{p}")));
    }
    if cnt0 == 0 || cnt1 == 0 || sum0 * cnt1 == sum1 * cnt0 {
        return Flow::Done;
    }
    let (winners, losers) = if sum0 * cnt1 > sum1 * cnt0 {
        (g0, g1)
    } else {
        (g1, g0)
    };
    let mut lmask = 0u32;
    for l in losers {
        if ctx.core.player(l).alive {
            lmask |= 1u32 << l;
        }
    }
    set_flag(ctx.core, owner, "dddfenye_losers_mask", lmask);
    m.targets = winners;
    dddfenye_next_winner(m, ctx)
}

fn dddfenye_next_winner(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    let lmask = flag(ctx.core, owner, "dddfenye_losers_mask");
    while let Some(&w) = m.targets.first() {
        if !ctx.core.player(w).alive {
            m.targets.remove(0);
            continue;
        }
        let has_target = ctx.core.alive_players().any(|p| {
            (lmask & (1u32 << p)) != 0
                && !is_sha_target_blocked(ctx.core, ctx.catalog, w, p, CardKind::Sha, DamageNature::Normal)
        });
        if has_target {
            return u01_wait_ask(m, ctx, HostStage::ShuchuiTargetLoop, 3019);
        }
        m.targets.remove(0);
    }
    Flow::Done
}

fn step_dddshichao(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    if m.card == Some(0) {
        match m.stage {
            HostStage::Act => {
                ctx.core
                    .vm_aux
                    .flags
                    .retain(|(o, k), _| !(*o == owner && k.starts_with("dddshichao_guard:")));
                u01_wait_ask(m, ctx, HostStage::TargetChoice, 3020)
            }
            HostStage::TargetChoice => {
                u01_clear_ask(ctx, owner);
                let Some(ChoiceOption::Player(t)) = ctx.answer().first().copied() else {
                    return Flow::Done;
                };
                let Some(lord) = u01_real_lord_seat(ctx.core) else {
                    return Flow::Done;
                };
                m.target = Some(t);
                let want = ctx
                    .core
                    .player(t)
                    .hand
                    .len()
                    .min(ctx.core.player(lord).max_hp as usize);
                let cur = ctx.core.player(owner).hand.len();
                set_flag(ctx.core, owner, &format!("dddshichao_guard:{t}"), 1);
                set_flag(ctx.core, owner, "dddshichao_just_armed", 1);
                ctx.log(LogEvent::SkillTriggered {
                    skill: "dddshichao",
                    owner,
                    presentation: None,
                });
                if cur < want {
                    ctx.start_draw(owner, (want - cur) as u32);
                    m.stage = HostStage::DrawSettle;
                    Flow::Yield
                } else if cur > want {
                    set_flag(ctx.core, owner, "dddshichao_discard_need", (cur - want) as u32);
                    u01_wait_ask(m, ctx, HostStage::SelectCard, 3021)
                } else {
                    Flow::Done
                }
            }
            HostStage::SelectCard => {
                u01_clear_ask(ctx, owner);
                ctx.core
                    .vm_aux
                    .flags
                    .remove(&(owner, "dddshichao_discard_need".to_string()));
                let drops: Vec<CardId> = ctx
                    .answer()
                    .iter()
                    .filter_map(|o| match o {
                        ChoiceOption::Card(c) => Some(*c),
                        _ => None,
                    })
                    .collect();
                if !drops.is_empty() {
                    u01_discard_batch(ctx, owner, &drops);
                }
                Flow::Done
            }
            _ => Flow::Done,
        }
    } else {
        match m.stage {
            HostStage::Act => u01_wait_ask(m, ctx, HostStage::ShenmiguoRestage, 3022),
            HostStage::ShenmiguoRestage => {
                u01_clear_ask(ctx, owner);
                if matches!(ctx.answer().first(), Some(ChoiceOption::Confirm)) {
                    for f in ctx.res.stack.iter_mut().rev() {
                        if let EventData::Damage(d) = &mut f.data {
                            if d.target == owner && !d.prevented {
                                d.prevented = true;
                                d.amount = 0;
                                break;
                            }
                        }
                    }
                    let mark = flag(ctx.core, owner, "dddshichao_mark") + 1;
                    set_flag(ctx.core, owner, "dddshichao_mark", mark);
                    ctx.log(LogEvent::SkillTriggered {
                        skill: "dddshichao",
                        owner,
                        presentation: None,
                    });
                }
                Flow::Done
            }
            _ => Flow::Done,
        }
    }
}
