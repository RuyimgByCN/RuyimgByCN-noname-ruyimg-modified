use super::*;

pub(super) fn step_u01_fix3_d1(machine: &mut HostMachine, ctx: &mut Ctx<'_>) -> Option<Flow> {
    let f = match machine.def_key.as_str() {
        "dddyouxue" => step_dddyouxue(machine, ctx),
        "dddchengjing" => step_dddchengjing(machine, ctx),
        "dddduanbing" => step_dddduanbing(machine, ctx),
        "dddxiaoxing" => step_dddxiaoxing(machine, ctx),
        "dddfuyi" => Flow::Done,
        "dddzhuanshe" => step_dddzhuanshe(machine, ctx),
        "dddweiqiu" => step_dddweiqiu(machine, ctx),
        "dddlianer" => step_dddlianer(machine, ctx),
        _ => return None,
    };
    Some(f)
}

fn step_dddyouxue(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.stage {
        HostStage::Act => {
            let r = ctx.core.turn.round.max(1);
            set_flag(ctx.core, owner, "dddyouxue_last_round", r);
            if global_youxue_holder(ctx.core).is_none() {
                set_global_youxue_holder(ctx.core, owner);
            }
            u01_wait_ask(m, ctx, HostStage::TargetChoice, 3040)
        }
        HostStage::TargetChoice => {
            u01_clear_ask(ctx, owner);
            let Some(ChoiceOption::Player(t)) = ctx.answer().first().copied() else {
                return Flow::Done;
            };
            let source = global_youxue_holder(ctx.core).unwrap_or(owner);
            set_flag(ctx.core, owner, "dddyouxue_holder", u32::from(t) + 1);
            set_global_youxue_holder(ctx.core, t);
            let dist = ctx.core.computed_distance(ctx.catalog, source, t).max(1) as u32;
            ctx.log(LogEvent::SkillTriggered {
                skill: "dddyouxue",
                owner,
                presentation: None,
            });
            ctx.start_draw(owner, dist);
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn step_dddchengjing(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    // Triggered at `TurnEndWindow { after: false }` (`m.card == None`) -> store last qualifying card into pile.
    // Or activated in Play phase (`m.card == Some(0)`) -> convert 1 `he` card as pile card's kind.
    // Or `useCardAfter` intrinsic park (`m.target == Some(dest)`, `m.card == Some(cid)`) -> give card to dest.
    if m.card.is_none() {
        let holder = match global_youxue_holder(ctx.core) {
            Some(h) => h,
            None => return Flow::Done,
        };
        if let Some(cid) = dddchengjing_last_qualifying_card(ctx.core, ctx.catalog, holder) {
            ctx.core.discard.retain(|&x| x != cid);
            if let Some(old) = ctx
                .core
                .vm_aux
                .piles
                .insert((owner, "dddchengjing".to_string()), vec![cid])
            {
                for oc in old {
                    ctx.to_discard(oc);
                }
            }
            ctx.log(LogEvent::SkillTriggered {
                skill: "dddchengjing",
                owner,
                presentation: None,
            });
        }
        return Flow::Done;
    }
    if m.target.is_some() {
        match m.stage {
            HostStage::Act => u01_wait_ask(m, ctx, HostStage::ShenmiguoRestage, 3043),
            HostStage::ShenmiguoRestage => {
                u01_clear_ask(ctx, owner);
                let Some(dest) = m.target else {
                    return Flow::Done;
                };
                let Some(cid) = m.card else {
                    return Flow::Done;
                };
                if matches!(ctx.answer().first(), Some(ChoiceOption::Confirm))
                    && ctx.core.player(dest).alive
                {
                    if ctx.core.discard.contains(&cid) {
                        ctx.core.discard.retain(|&x| x != cid);
                        ctx.core.declared_kinds.remove(&cid);
                        ctx.core.player_mut(dest).hand.push(cid);
                        ctx.log(LogEvent::CardsGiven {
                            from: owner,
                            to: dest,
                            cards: vec![cid],
                        });
                    } else if ctx.core.table.contains(&cid) {
                        ctx.core.table.retain(|&x| x != cid);
                        ctx.core.declared_kinds.remove(&cid);
                        ctx.core.player_mut(dest).hand.push(cid);
                        ctx.log(LogEvent::CardsGiven {
                            from: owner,
                            to: dest,
                            cards: vec![cid],
                        });
                    }
                } else {
                    if ctx.core.table.contains(&cid) {
                        ctx.core.table.retain(|&x| x != cid);
                        ctx.to_discard(cid);
                    }
                    ctx.core.declared_kinds.remove(&cid);
                }
                Flow::Done
            }
            _ => Flow::Done,
        }
    } else {
        match m.stage {
            HostStage::Act => u01_wait_ask(m, ctx, HostStage::SelectCard, 3041),
            HostStage::SelectCard => {
                u01_clear_ask(ctx, owner);
                let Some(ChoiceOption::Card(picked)) = ctx.answer().first().copied() else {
                    return Flow::Done;
                };
                let Some(&pile_c) = ctx
                    .core
                    .vm_aux
                    .piles
                    .get(&(owner, "dddchengjing".to_string()))
                    .and_then(|v| v.first())
                else {
                    return Flow::Done;
                };
                m.cards = vec![picked];
                let as_kind = ctx.catalog.card_kind(pile_c);
                let needs_target = !matches!(
                    as_kind,
                    CardKind::Tao
                        | CardKind::Jiu
                        | CardKind::Wuzhong
                        | CardKind::Taoyuan
                        | CardKind::Wugu
                        | CardKind::SavageAttack
                        | CardKind::ArcheryAttack
                        | CardKind::Le
                        | CardKind::Bing
                        | CardKind::ShanDian
                ) && !as_kind.is_equip();
                if needs_target {
                    let legals = virtual_legal_targets(ctx.core, ctx.catalog, owner, as_kind);
                    if legals.is_empty() {
                        return Flow::Done;
                    }
                    u01_wait_ask(m, ctx, HostStage::TargetChoice, 3042)
                } else {
                    if matches!(as_kind, CardKind::Tao | CardKind::Jiu | CardKind::Taoyuan)
                        && virtual_legal_targets(ctx.core, ctx.catalog, owner, as_kind).is_empty()
                    {
                        return Flow::Done;
                    }
                    dddchengjing_emit_use(ctx, owner, picked, as_kind, vec![owner])
                }
            }
            HostStage::TargetChoice => {
                u01_clear_ask(ctx, owner);
                let Some(ChoiceOption::Player(tgt)) = ctx.answer().first().copied() else {
                    return Flow::Done;
                };
                let Some(&picked) = m.cards.first() else {
                    return Flow::Done;
                };
                let Some(&pile_c) = ctx
                    .core
                    .vm_aux
                    .piles
                    .get(&(owner, "dddchengjing".to_string()))
                    .and_then(|v| v.first())
                else {
                    return Flow::Done;
                };
                let as_kind = ctx.catalog.card_kind(pile_c);
                if as_kind == CardKind::Jiedao {
                    m.targets = vec![tgt];
                    return u01_wait_ask(m, ctx, HostStage::CompareTargetPick, 3066);
                }
                dddchengjing_emit_use(ctx, owner, picked, as_kind, vec![tgt])
            }
            HostStage::CompareTargetPick => {
                u01_clear_ask(ctx, owner);
                let Some(ChoiceOption::Player(victim)) = ctx.answer().first().copied() else {
                    return Flow::Done;
                };
                let Some(&holder) = m.targets.first() else {
                    return Flow::Done;
                };
                let Some(&picked) = m.cards.first() else {
                    return Flow::Done;
                };
                dddchengjing_emit_use(ctx, owner, picked, CardKind::Jiedao, vec![holder, victim])
            }
            _ => Flow::Done,
        }
    }
}

fn dddchengjing_emit_use(
    ctx: &mut Ctx<'_>,
    owner: PlayerId,
    picked: CardId,
    as_kind: CardKind,
    targets: Vec<PlayerId>,
) -> Flow {
    let tok = play_phase_token(ctx.core);
    set_flag(ctx.core, owner, "dddchengjing_used_tok", tok);
    let final_targets = if matches!(
        as_kind,
        CardKind::Taoyuan | CardKind::Wugu | CardKind::SavageAttack | CardKind::ArcheryAttack
    ) {
        Vec::new()
    } else {
        targets
    };
    let nature = match as_kind {
        CardKind::FireSha | CardKind::Linghunzhihuo => DamageNature::Fire,
        CardKind::LeiSha => DamageNature::Thunder,
        _ => DamageNature::Normal,
    };
    ctx.emit(EventData::CardUsed(CardUseData {
        card: Some(picked),
        player: owner,
        targets: final_targets,
        nature,
        use_as: Some(as_kind),
        cancelled: false,
    }));
    if let Some(new_fid) = ctx.res.stack.last().map(|f| f.id) {
        set_flag(ctx.core, owner, &format!("dddchengjing_use_fid:{new_fid}"), 1);
    }
    ctx.log(LogEvent::SkillTriggered {
        skill: "dddchengjing",
        owner,
        presentation: None,
    });
    Flow::Done
}

fn step_dddduanbing(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    if m.card == Some(1) {
        // `dddduanbing_effect` (`DamageSource` by card): transfer ALL `Bing` judge cards (no duplicates on target).
        match m.stage {
            HostStage::Act => u01_wait_ask(m, ctx, HostStage::ShenmiguoRestage, 3046),
            HostStage::ShenmiguoRestage => {
                u01_clear_ask(ctx, owner);
                let Some(t) = m.target else {
                    return Flow::Done;
                };
                if matches!(ctx.answer().first(), Some(ChoiceOption::Confirm))
                    && ctx.core.player(t).alive
                {
                    let bings: Vec<CardId> = ctx
                        .core
                        .player(owner)
                        .judge
                        .iter()
                        .copied()
                        .filter(|&jc| eff_card_kind_3(ctx.core, ctx.catalog, jc) == CardKind::Bing)
                        .collect();
                    for jc in bings {
                        let target_has = ctx
                            .core
                            .player(t)
                            .judge
                            .iter()
                            .any(|&tc| eff_card_kind_3(ctx.core, ctx.catalog, tc) == CardKind::Bing);
                        if target_has {
                            break;
                        }
                        ctx.core.player_mut(owner).judge.retain(|&x| x != jc);
                        ctx.core.player_mut(t).judge.push(jc);
                        ctx.log(LogEvent::CardsGiven {
                            from: owner,
                            to: t,
                            cards: vec![jc],
                        });
                    }
                    ctx.log(LogEvent::SkillTriggered {
                        skill: "dddduanbing",
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
            HostStage::Act => u01_wait_ask(m, ctx, HostStage::SelectCard, 3044),
            HostStage::SelectCard => {
                u01_clear_ask(ctx, owner);
                let Some(ChoiceOption::Card(c)) = ctx.answer().first().copied() else {
                    return Flow::Done;
                };
                u01_remove_owned_card_quiet(ctx, owner, c);
                ctx.note_cards_lost(owner, &[c]);
                ctx.core.declared_kinds.insert(c, CardKind::Bing);
                ctx.core.player_mut(owner).judge.push(c);
                ctx.log(LogEvent::SkillTriggered {
                    skill: "dddduanbing",
                    owner,
                    presentation: None,
                });
                ctx.start_draw(owner, 2);
                m.stage = HostStage::DrawSettle;
                Flow::Yield
            }
            HostStage::DrawSettle => {
                let has_sha_tgt = ctx
                    .core
                    .alive_players()
                    .any(|p| p != owner && !is_sha_target_blocked(ctx.core, ctx.catalog, owner, p, CardKind::Sha, DamageNature::Normal));
                if has_sha_tgt {
                    u01_wait_ask(m, ctx, HostStage::TargetChoice, 3045)
                } else {
                    Flow::Done
                }
            }
            HostStage::TargetChoice => {
                u01_clear_ask(ctx, owner);
                if let Some(ChoiceOption::Player(t)) = ctx.answer().first().copied() {
                    ctx.emit(EventData::CardUsed(CardUseData {
                        card: None,
                        player: owner,
                        targets: vec![t],
                        nature: DamageNature::Normal,
                        use_as: Some(CardKind::Sha),
                        cancelled: false,
                    }));
                }
                Flow::Done
            }
            _ => Flow::Done,
        }
    }
}

fn step_dddxiaoxing(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    ctx.log(LogEvent::SkillTriggered {
        skill: "dddxiaoxing",
        owner,
        presentation: None,
    });
    if let Some(def) = ctx.catalog.skill_by_name("dddxiaoxing") {
        ctx.remove_skill(owner, def);
    }
    Flow::Done
}

fn step_dddzhuanshe(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.card {
        Some(0) => match m.stage {
            HostStage::Act => u01_wait_ask(m, ctx, HostStage::SelectCard, 3048),
            HostStage::SelectCard => {
                u01_clear_ask(ctx, owner);
                let Some(cur) = m.target else {
                    return Flow::Done;
                };
                if let Some(ChoiceOption::Card(c)) = ctx.answer().first().copied() {
                    let kind = ctx.catalog.card_kind(c);
                    let key_kind = dddzhuanshe_card_key(kind);
                    u01_give_card_ex(ctx, owner, cur, c, false);
                    let stamp = turn_stamp_3(ctx.core);
                    set_flag(
                        ctx.core,
                        owner,
                        &format!("dddzhuanshe_given:{stamp}:{key_kind}"),
                        1,
                    );
                    ctx.log(LogEvent::SkillTriggered {
                        skill: "dddzhuanshe",
                        owner,
                        presentation: None,
                    });
                }
                Flow::Done
            }
            _ => Flow::Done,
        },
        Some(1) => match m.stage {
            HostStage::Act => {
                let mut existing = Vec::new();
                for f in ctx.res.stack.iter().rev() {
                    if let EventData::CardUsed(u) = &f.data {
                        let eff = u.effective_kind(ctx.catalog);
                        let all = virtual_legal_targets(ctx.core, ctx.catalog, u.player, eff);
                        m.targets = all
                            .into_iter()
                            .filter(|p| !u.targets.contains(p))
                            .collect();
                        existing = u.targets.clone();
                        break;
                    }
                }
                let _ = existing;
                if m.targets.is_empty() {
                    Flow::Done
                } else {
                    u01_wait_ask(m, ctx, HostStage::TargetChoice, 3049)
                }
            }
            HostStage::TargetChoice => {
                u01_clear_ask(ctx, owner);
                if let Some(ChoiceOption::Player(add_t)) = ctx.answer().first().copied() {
                    for f in ctx.res.stack.iter_mut().rev() {
                        if let EventData::CardUsed(u) = &mut f.data {
                            if !u.targets.contains(&add_t) {
                                u.targets.push(add_t);
                            }
                            break;
                        }
                    }
                    ctx.log(LogEvent::SkillTriggered {
                        skill: "dddzhuanshe",
                        owner,
                        presentation: None,
                    });
                }
                Flow::Done
            }
            _ => Flow::Done,
        },
        _ => match m.stage {
            HostStage::Act => u01_wait_ask(m, ctx, HostStage::ShenmiguoRestage, 3050),
            HostStage::ShenmiguoRestage => {
                u01_clear_ask(ctx, owner);
                let Some(t) = m.target else {
                    return Flow::Done;
                };
                if matches!(ctx.answer().first(), Some(ChoiceOption::Confirm))
                    && ctx.core.player(t).alive
                {
                    ctx.emit(EventData::Damage(damage_data_with(
                        Some(owner),
                        t,
                        1,
                        DamageNature::Normal,
                        None,
                    )));
                    ctx.log(LogEvent::SkillTriggered {
                        skill: "dddzhuanshe",
                        owner,
                        presentation: None,
                    });
                }
                Flow::Done
            }
            _ => Flow::Done,
        },
    }
}

fn step_dddweiqiu(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.stage {
        HostStage::Act => u01_wait_ask(m, ctx, HostStage::ShenmiguoRestage, 3051),
        HostStage::ShenmiguoRestage => {
            u01_clear_ask(ctx, owner);
            if matches!(ctx.answer().first(), Some(ChoiceOption::Confirm))
                && ctx.core.player(owner).hand.is_empty()
            {
                if let Some(fid) = ctx
                    .res
                    .stack
                    .iter()
                    .rev()
                    .find(|f| matches!(f.data, EventData::Recover { .. }))
                    .map(|f| f.id)
                {
                    set_flag(ctx.core, 0, &format!("u01_recover_cancelled:{fid}"), 1);
                }
                ctx.log(LogEvent::SkillTriggered {
                    skill: "dddweiqiu",
                    owner,
                    presentation: None,
                });
                ctx.start_draw(owner, 1);
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn step_dddlianer(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.stage {
        HostStage::Act => u01_wait_ask(m, ctx, HostStage::ShenmiguoRestage, 3052),
        HostStage::ShenmiguoRestage => {
            u01_clear_ask(ctx, owner);
            if matches!(ctx.answer().first(), Some(ChoiceOption::Confirm)) {
                if let Some(cid) = m.card {
                    if ctx.core.discard.contains(&cid) {
                        ctx.core.discard.retain(|&x| x != cid);
                        ctx.core.player_mut(owner).hand.push(cid);
                        ctx.log(LogEvent::CardsDrawn {
                            player: owner,
                            cards: vec![cid],
                        });
                        let rank = u32::from(ctx.catalog.cards[cid as usize].rank);
                        if rank > 0 {
                            set_flag(ctx.core, owner, "dddlianer_rank_cap", rank);
                        }
                        ctx.log(LogEvent::SkillTriggered {
                            skill: "dddlianer",
                            owner,
                            presentation: None,
                        });
                    }
                }
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}
