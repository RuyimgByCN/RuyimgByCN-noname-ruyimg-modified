//! Step handlers for Batch B part 1:
//! `dddzhengjun`, `dddxianxi`, `dddlingyong`, `dddxuxiao`, `dddxuyu`.

use super::*;

pub(super) fn step_b1(machine: &mut HostMachine, ctx: &mut Ctx<'_>) -> Option<Flow> {
    match machine.def_key.as_str() {
        "dddzhengjun" => Some(step_dddzhengjun(machine, ctx)),
        "dddxianxi" => Some(step_dddxianxi(machine, ctx)),
        "dddlingyong" => Some(step_dddlingyong(machine, ctx)),
        "dddxuxiao" => Some(step_dddxuxiao(machine, ctx)),
        "dddxuyu" if machine.target.is_some() => {
            Some(step_dddxuyu_when_discard(machine, ctx))
        }
        "dddxuyu" => Some(step_dddxuyu(machine, ctx)),
        _ => None,
    }
}

fn step_dddzhengjun(machine: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = machine.owner;
    let branch = machine.card.unwrap_or(0) as u8;
    if !ctx.core.player(owner).alive
        || ctx.core.turn.current != owner
        || ctx.core.turn.phase != Phase::Play
        || ctx.catalog.is_tieji_sealed(ctx.core, owner)
    {
        return Flow::Done;
    }
    let play_seq = flag(ctx.core, 0, "play_phase_seq").max(1);
    let used_key = format!("dddzhengjun_used_b{branch}");
    match machine.stage {
        HostStage::Act => {
            if flag(ctx.core, owner, &used_key) == play_seq {
                return Flow::Done;
            }
            let affected = machine.target.unwrap_or(owner);
            let matches = dddzhengjun_match_targets(
                ctx.core,
                owner,
                affected,
                &machine.targets,
                branch,
            );
            if matches.is_empty() {
                return Flow::Done;
            }
            match branch {
                1 => {
                    if matches.len() == 1 {
                        machine.target = Some(matches[0]);
                        wait_ask_2(machine, ctx, HostStage::TargetChoice, 2001)
                    } else {
                        wait_ask_2(machine, ctx, HostStage::SelectNature, 2007)
                    }
                }
                2 => {
                    if matches.len() == 1 {
                        machine.target = Some(matches[0]);
                        wait_ask_2(machine, ctx, HostStage::TargetChoice, 2002)
                    } else {
                        wait_ask_2(machine, ctx, HostStage::SelectNature, 2008)
                    }
                }
                3 => wait_ask_2(machine, ctx, HostStage::TargetChoice, 2004),
                _ => Flow::Done,
            }
        }
        HostStage::SelectNature => {
            let Some(ChoiceOption::Player(t)) = ctx.answer().first().copied() else {
                return Flow::Done;
            };
            machine.target = Some(t);
            let ask_id = if branch == 1 { 2001 } else { 2002 };
            wait_ask_2(machine, ctx, HostStage::TargetChoice, ask_id)
        }
        HostStage::TargetChoice => {
            if flag(ctx.core, owner, &used_key) == play_seq {
                return Flow::Done;
            }
            match branch {
                1 => {
                    let Some(victim) = machine.target else {
                        return Flow::Done;
                    };
                    let Some(ChoiceOption::Tag(tag)) = ctx.answer().first().copied() else {
                        return Flow::Done;
                    };
                    if !ctx.core.player(victim).alive {
                        return Flow::Done;
                    }
                    set_flag(ctx.core, owner, &used_key, play_seq);
                    ctx.log(LogEvent::SkillTriggered {
                        skill: "dddzhengjun",
                        owner,
                        presentation: None,
                    });
                    if tag == 0 {
                        ctx.recover(victim, 1, None);
                    } else {
                        ctx.emit(EventData::LoseHp {
                            player: victim,
                            amount: 1,
                        });
                    }
                    Flow::Done
                }
                2 => {
                    let Some(victim) = machine.target else {
                        return Flow::Done;
                    };
                    let Some(ChoiceOption::Tag(tag)) = ctx.answer().first().copied() else {
                        return Flow::Done;
                    };
                    if !ctx.core.player(victim).alive {
                        return Flow::Done;
                    }
                    set_flag(ctx.core, owner, &used_key, play_seq);
                    ctx.log(LogEvent::SkillTriggered {
                        skill: "dddzhengjun",
                        owner,
                        presentation: None,
                    });
                    if tag == 0 {
                        ctx.start_draw(victim, 1);
                        Flow::Done
                    } else if he_cards(ctx.core, victim).is_empty() {
                        Flow::Done
                    } else {
                        wait_ask_2(machine, ctx, HostStage::SelectCard, 2003)
                    }
                }
                3 => {
                    if !matches!(ctx.answer().first(), Some(ChoiceOption::Confirm)) {
                        return Flow::Done;
                    }
                    set_flag(ctx.core, owner, &used_key, play_seq);
                    ctx.log(LogEvent::SkillTriggered {
                        skill: "dddzhengjun",
                        owner,
                        presentation: None,
                    });
                    let Some(es_target) = machine.target else {
                        return Flow::Done;
                    };
                    if dddzhengjun_movable_field_cards(ctx.core, ctx.catalog, es_target).is_empty()
                    {
                        return Flow::Done;
                    }
                    wait_ask_2(machine, ctx, HostStage::SelectCard, 2005)
                }
                _ => Flow::Done,
            }
        }
        HostStage::SelectCard => match branch {
            2 => {
                let Some(victim) = machine.target else {
                    return Flow::Done;
                };
                if let Some(ChoiceOption::Card(c)) = ctx.answer().first().copied() {
                    u01_discard_batch(ctx, victim, &[c]);
                }
                Flow::Done
            }
            3 => {
                let Some(ChoiceOption::Card(c)) = ctx.answer().first().copied() else {
                    return Flow::Done;
                };
                machine.card = Some(c);
                wait_ask_2(machine, ctx, HostStage::CardsAsk, 2006)
            }
            _ => Flow::Done,
        },
        HostStage::CardsAsk => {
            let Some(move_card) = machine.card else {
                return Flow::Done;
            };
            let Some(ChoiceOption::Player(dest)) = ctx.answer().first().copied() else {
                return Flow::Done;
            };
            let Some(src) = machine.target.filter(|&p| ctx.core.player(p).alive) else {
                return Flow::Done;
            };
            if ctx.core.player(src).equips.contains(&move_card) {
                ctx.core
                    .player_mut(src)
                    .equips
                    .retain(|&x| x != move_card);
                ctx.core.declared_slots.remove(&move_card);
                crate::kernel::revoke_equip_skills(ctx.core, ctx.catalog, src, move_card);
                ctx.emit(EventData::CardsLost {
                    player: src,
                    cards: vec![move_card],
                    origins: vec![LostOrigin::Equip],
                });
                ctx.core.player_mut(dest).equips.push(move_card);
                crate::kernel::grant_equip_skills(ctx, dest, move_card);
                ctx.emit(EventData::CardsMoved {
                    from: src,
                    to: dest,
                    cards: vec![move_card],
                });
            } else if ctx.core.player(src).judge.contains(&move_card) {
                ctx.core
                    .player_mut(src)
                    .judge
                    .retain(|&x| x != move_card);
                ctx.emit(EventData::CardsLost {
                    player: src,
                    cards: vec![move_card],
                    origins: vec![LostOrigin::Judge],
                });
                ctx.core.player_mut(dest).judge.push(move_card);
                ctx.emit(EventData::CardsMoved {
                    from: src,
                    to: dest,
                    cards: vec![move_card],
                });
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn step_dddxianxi(machine: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = machine.owner;
    if !ctx.core.player(owner).alive {
        return Flow::Done;
    }
    let is_delay = machine.card == Some(10);
    if !is_delay {
        match machine.stage {
            HostStage::Act => {
                if ctx.core.turn.current != owner
                    || ctx.core.turn.phase != Phase::Play
                    || ctx.catalog.is_tieji_sealed(ctx.core, owner)
                {
                    return Flow::Done;
                }
                let nature = match ctx.res.stack.iter().rev().find_map(|f| match &f.data {
                    EventData::CardUsed(u) => Some(u.nature),
                    _ => None,
                }) {
                    Some(DamageNature::Fire) => 1,
                    Some(DamageNature::Thunder) => 2,
                    _ => 0,
                };
                machine.card = Some(nature);
                wait_ask_2(machine, ctx, HostStage::TargetChoice, 2009)
            }
            HostStage::TargetChoice => {
                let mut added: Vec<PlayerId> = ctx
                    .answer()
                    .iter()
                    .filter_map(|opt| match opt {
                        ChoiceOption::Player(p) => Some(*p),
                        _ => None,
                    })
                    .collect();
                if added.is_empty() {
                    return Flow::Done;
                }
                added.sort_unstable();
                let cur = ctx.core.turn.current;
                let n = ctx.core.players.len() as PlayerId;
                added.sort_by_key(|&p| (p + n - cur) % n);
                let mut add_mask = 0u32;
                for &p in &added {
                    add_mask |= 1u32 << p;
                }
                if let Some(sha_frame) = ctx
                    .res
                    .stack
                    .iter_mut()
                    .rev()
                    .find(|f| matches!(f.data, EventData::CardUsed(_)))
                {
                    let sha_fid = sha_frame.id;
                    if let EventData::CardUsed(u) = &mut sha_frame.data {
                        for &p in &added {
                            if !u.targets.contains(&p) {
                                u.targets.push(p);
                            }
                        }
                        u.targets.sort_by_key(|&p| (p + n - cur) % n);
                    }
                    set_flag(ctx.core, owner, &format!("dddxianxi_added:{sha_fid}"), add_mask);
                }
                ctx.log(LogEvent::SkillTriggered {
                    skill: "dddxianxi",
                    owner,
                    presentation: None,
                });
                Flow::Done
            }
            _ => Flow::Done,
        }
    } else {
        match machine.stage {
            HostStage::Act => {
                let num = machine.targets.len();
                if num == 0 {
                    return Flow::Done;
                }
                ctx.log(LogEvent::SkillTriggered {
                    skill: "dddxianxi",
                    owner,
                    presentation: None,
                });
                if he_cards(ctx.core, owner).len() < num {
                    ctx.start_draw(owner, num as u32);
                    ctx.emit(EventData::LoseHp {
                        player: owner,
                        amount: 1,
                    });
                    return Flow::Done;
                }
                wait_ask_2(machine, ctx, HostStage::SelectCard, 2010)
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
                let num = machine.targets.len();
                if picks.len() == num {
                    u01_discard_batch(ctx, owner, &picks);
                    let targets = machine.targets.clone();
                    for t in targets {
                        if ctx.core.player(t).alive {
                            ctx.emit(EventData::Damage(damage_data_with(
                                Some(owner),
                                t,
                                1,
                                DamageNature::Normal,
                                None,
                            )));
                        }
                    }
                } else {
                    ctx.start_draw(owner, num as u32);
                    ctx.emit(EventData::LoseHp {
                        player: owner,
                        amount: 1,
                    });
                }
                Flow::Done
            }
            _ => Flow::Done,
        }
    }
}

fn step_dddlingyong(machine: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = machine.owner;
    if !ctx.core.player(owner).alive || ctx.catalog.is_tieji_sealed(ctx.core, owner) {
        return Flow::Done;
    }
    match machine.stage {
        HostStage::Act => wait_ask_2(machine, ctx, HostStage::TargetChoice, 2011),
        HostStage::TargetChoice | HostStage::SelectNature => {
            if !matches!(ctx.answer().first(), Some(ChoiceOption::Confirm)) {
                return Flow::Done;
            }
            if ctx.core.deck.is_empty() && ctx.core.discard.is_empty() {
                return Flow::Done;
            }
            ctx.log(LogEvent::SkillTriggered {
                skill: "dddlingyong",
                owner,
                presentation: None,
            });
            ctx.core
                .vm_aux
                .flags
                .remove(&(owner, "last_judge_card".to_string()));
            machine.stage = HostStage::DrawSettle;
            ctx.emit(EventData::OrdinaryJudge {
                owner,
                invocation: u32::MAX,
            });
            Flow::Yield
        }
        HostStage::DrawSettle => {
            let Some(card) = ctx
                .core
                .vm_aux
                .flags
                .remove(&(owner, "last_judge_card".to_string()))
            else {
                return Flow::Done;
            };
            if ctx.catalog.card_kind(card).is_sha_like() {
                return Flow::Done;
            }
            machine.card = Some(card);
            if ctx.core.discard.contains(&card)
                && dddlingyong_targets(ctx.core, ctx.catalog, owner, card).is_some()
            {
                return wait_ask_2(machine, ctx, HostStage::SelectCard, 2012);
            }
            wait_ask_2(machine, ctx, HostStage::SelectNature, 2013)
        }
        HostStage::SelectCard => {
            let Some(card) = machine.card else {
                return Flow::Done;
            };
            let kind = ctx.catalog.card_kind(card);
            let ans = ctx.answer().clone();
            let chosen_targets: Option<Vec<PlayerId>> = if dddlingyong_needs_target_prompt(ctx.catalog, card) {
                if let Some(ChoiceOption::Player(t)) = ans.first().copied() {
                    if kind == CardKind::Jiedao {
                        machine.target = Some(t);
                        return wait_ask_2(machine, ctx, HostStage::JiefenTarget, 2051);
                    }
                    Some(vec![t])
                } else {
                    None
                }
            } else if matches!(ans.first(), Some(ChoiceOption::Confirm)) {
                Some(dddlingyong_targets(ctx.core, ctx.catalog, owner, card).unwrap_or_default())
            } else {
                None
            };
            if let Some(tgts) = chosen_targets {
                return emit_lingyong_card(machine, ctx, owner, card, kind, tgts);
            }
            wait_ask_2(machine, ctx, HostStage::SelectNature, 2013)
        }
        HostStage::JiefenTarget => {
            let Some(card) = machine.card else {
                return Flow::Done;
            };
            let Some(holder) = machine.target else {
                return Flow::Done;
            };
            let Some(ChoiceOption::Player(victim)) = ctx.answer().first().copied() else {
                return wait_ask_2(machine, ctx, HostStage::SelectNature, 2013);
            };
            emit_lingyong_card(
                machine,
                ctx,
                owner,
                card,
                CardKind::Jiedao,
                vec![holder, victim],
            )
        }
        HostStage::CardsAsk => wait_ask_2(machine, ctx, HostStage::SelectNature, 2013),
        _ => Flow::Done,
    }
}

fn emit_lingyong_card(
    machine: &mut HostMachine,
    ctx: &mut Ctx<'_>,
    owner: PlayerId,
    card: CardId,
    kind: CardKind,
    targets: Vec<PlayerId>,
) -> Flow {
    if ctx.core.discard.contains(&card) {
        ctx.core.discard.retain(|&x| x != card);
        ctx.core.table.push(card);
    }
    let nature = match kind {
        CardKind::FireSha | CardKind::Linghunzhihuo => DamageNature::Fire,
        CardKind::LeiSha => DamageNature::Thunder,
        _ => DamageNature::Normal,
    };
    let next_fid = ctx.res.next_event_id;
    if kind.is_equip() {
        set_flag(ctx.core, owner, &format!("u01_equip_from_table:{next_fid}"), 1);
        if ctx.catalog.skill_by_name("dddchashi").is_some() {
            set_flag(ctx.core, owner, &format!("chashi_equip_use:{next_fid}"), 1);
        }
        machine.stage = HostStage::CardsAsk;
        ctx.emit(EventData::Equipped {
            player: owner,
            card,
            replaced: None,
        });
        return Flow::Yield;
    }
    machine.stage = HostStage::CardsAsk;
    ctx.emit(EventData::CardUsed(CardUseData {
        card: Some(card),
        player: owner,
        targets,
        nature,
        use_as: None,
        cancelled: false,
    }));
    Flow::Yield
}

fn step_dddxuxiao(machine: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = machine.owner;
    if !ctx.core.player(owner).alive || ctx.catalog.is_tieji_sealed(ctx.core, owner) {
        return Flow::Done;
    }
    match machine.stage {
        HostStage::Act => {
            if dddxuxiao_valid_cards(ctx.core, ctx.catalog, &machine.cards).is_empty() {
                return Flow::Done;
            }
            wait_ask_2(machine, ctx, HostStage::SelectCard, 2014)
        }
        HostStage::SelectCard => {
            let Some(ChoiceOption::Card(c)) = ctx.answer().first().copied() else {
                return Flow::Done;
            };
            if dddxuxiao_valid_targets(ctx.core, ctx.catalog).is_empty() {
                return Flow::Done;
            }
            machine.card = Some(c);
            machine.cards.retain(|&x| x != c);
            wait_ask_2(machine, ctx, HostStage::TargetChoice, 2015)
        }
        HostStage::TargetChoice => {
            let Some(c) = machine.card else {
                return Flow::Done;
            };
            let Some(ChoiceOption::Player(t)) = ctx.answer().first().copied() else {
                return Flow::Done;
            };
            if !ctx.core.discard.contains(&c) || !ctx.core.player(t).alive {
                return Flow::Done;
            }
            ctx.core.discard.retain(|&x| x != c);
            ctx.core.declared_kinds.insert(c, CardKind::Bing);
            ctx.core.player_mut(t).judge.push(c);
            ctx.log(LogEvent::SkillTriggered {
                skill: "dddxuxiao",
                owner,
                presentation: None,
            });
            ctx.start_draw(t, 2);
            machine.card = None;
            machine.stage = HostStage::DrawSettle;
            Flow::Yield
        }
        HostStage::DrawSettle => {
            if dddxuxiao_valid_cards(ctx.core, ctx.catalog, &machine.cards).is_empty() {
                return Flow::Done;
            }
            wait_ask_2(machine, ctx, HostStage::SelectCard, 2014)
        }
        _ => Flow::Done,
    }
}

fn step_dddxuyu(machine: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = machine.owner;
    if !ctx.core.player(owner).alive || ctx.catalog.is_tieji_sealed(ctx.core, owner) {
        return Flow::Done;
    }
    match machine.stage {
        HostStage::Act => wait_ask_2(machine, ctx, HostStage::TargetChoice, 2016),
        HostStage::TargetChoice => {
            let fid = ctx
                .core
                .vm_aux
                .flags
                .remove(&(owner, "dddxuyu_next_fid".to_string()))
                .unwrap_or(1)
                .max(1);
            if !matches!(ctx.answer().first(), Some(ChoiceOption::Confirm)) {
                return Flow::Done;
            }
            ctx.log(LogEvent::SkillTriggered {
                skill: "dddxuyu",
                owner,
                presentation: None,
            });
            set_flag(ctx.core, owner, "dddxuyu_pending_fid", fid);
            ctx.start_draw(owner, 1);
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn step_dddxuyu_when_discard(machine: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = machine.owner;
    if !ctx.core.player(owner).alive {
        return Flow::Done;
    }
    match machine.stage {
        HostStage::Act => {
            if he_cards(ctx.core, owner).is_empty() {
                return Flow::Done;
            }
            wait_ask_2(machine, ctx, HostStage::SelectCard, 2017)
        }
        HostStage::SelectCard => {
            if let Some(ChoiceOption::Card(c)) = ctx.answer().first().copied() {
                u01_discard_batch(ctx, owner, &[c]);
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}
