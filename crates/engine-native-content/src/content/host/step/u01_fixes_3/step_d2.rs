use super::*;

pub(super) fn step_u01_fix3_d2(machine: &mut HostMachine, ctx: &mut Ctx<'_>) -> Option<Flow> {
    let f = match machine.def_key.as_str() {
        "dddshixing" => {
            dddshixing_apply(ctx, machine.owner);
            Flow::Done
        }
        "xiangle" => step_xiangle(machine, ctx),
        "jiushi" => step_jiushi(machine, ctx),
        "rezhiheng" => step_rezhiheng(machine, ctx),
        "ddddanggu" => step_ddddanggu(machine, ctx),
        "dddlanghuai" => step_dddlanghuai(machine, ctx),
        "dddxuanlun" => step_dddxuanlun(machine, ctx),
        "dddjiexing" => step_dddjiexing(machine, ctx),
        "dddbailei" => step_dddbailei(machine, ctx),
        _ => return None,
    };
    Some(f)
}

fn step_xiangle(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.stage {
        HostStage::Act => {
            let Some(sha_user) = m.target else {
                return Flow::Done;
            };
            ctx.log(LogEvent::SkillTriggered {
                skill: "xiangle",
                owner,
                presentation: None,
            });
            let has_basic = ctx
                .core
                .player(sha_user)
                .hand
                .iter()
                .any(|&c| eff_card_kind_3(ctx.core, ctx.catalog, c).is_basic_kind());
            if !has_basic {
                exclude_self_from_active_sha(ctx, owner);
                return Flow::Done;
            }
            u01_wait_ask(m, ctx, HostStage::SelectCard, 3053)
        }
        HostStage::SelectCard => {
            u01_clear_ask(ctx, owner);
            let Some(sha_user) = m.target else {
                return Flow::Done;
            };
            if let Some(ChoiceOption::Card(c)) = ctx.answer().first().copied() {
                u01_discard_batch(ctx, sha_user, &[c]);
            } else {
                exclude_self_from_active_sha(ctx, owner);
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn exclude_self_from_active_sha(ctx: &mut Ctx<'_>, owner: PlayerId) {
    for f in ctx.res.stack.iter_mut().rev() {
        if let EventData::CardUsed(u) = &mut f.data {
            u.targets.retain(|&t| t != owner);
            if u.targets.is_empty() {
                u.cancelled = true;
            }
            break;
        }
    }
}

fn step_jiushi(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    if m.card == Some(0) {
        // Active Play phase: turn face-down + virtual `Jiu` (`jiu_active`).
        ctx.emit(EventData::CardUsed(CardUseData {
            card: None,
            player: owner,
            targets: vec![owner],
            nature: DamageNature::Normal,
            use_as: Some(CardKind::Jiu),
            cancelled: false,
        }));
        ctx.emit(EventData::TurnOverBefore {
            player: owner,
            cancelled: false,
        });
        ctx.log(LogEvent::SkillTriggered {
            skill: "jiushi",
            owner,
            presentation: None,
        });
        Flow::Done
    } else {
        match m.stage {
            HostStage::Act => u01_wait_ask(m, ctx, HostStage::ShenmiguoRestage, 3054),
            HostStage::ShenmiguoRestage => {
                u01_clear_ask(ctx, owner);
                if matches!(ctx.answer().first(), Some(ChoiceOption::Confirm))
                    && ctx.core.player(owner).face_down
                {
                    ctx.emit(EventData::TurnOverBefore {
                        player: owner,
                        cancelled: false,
                    });
                    ctx.log(LogEvent::SkillTriggered {
                        skill: "jiushi",
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

fn step_rezhiheng(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.stage {
        HostStage::Act => u01_wait_ask(m, ctx, HostStage::SelectCard, 3055),
        HostStage::SelectCard => {
            u01_clear_ask(ctx, owner);
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
            set_flag(ctx.core, owner, "rezhiheng_used_tok", play_phase_token(ctx.core));
            let had_hand = !ctx.core.player(owner).hand.is_empty();
            let all_hand_discarded = had_hand
                && ctx
                    .core
                    .player(owner)
                    .hand
                    .iter()
                    .all(|h| picks.contains(h));
            u01_discard_batch(ctx, owner, &picks);
            let draw_n = picks.len() as u32 + u32::from(all_hand_discarded);
            ctx.log(LogEvent::SkillTriggered {
                skill: "rezhiheng",
                owner,
                presentation: None,
            });
            ctx.start_draw(owner, draw_n);
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn step_ddddanggu(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.stage {
        HostStage::Act => u01_wait_ask(m, ctx, HostStage::TargetChoice, 3056),
        HostStage::TargetChoice => {
            u01_clear_ask(ctx, owner);
            let chosen: Vec<PlayerId> = ctx
                .answer()
                .iter()
                .filter_map(|o| match o {
                    ChoiceOption::Player(p) => Some(*p),
                    _ => None,
                })
                .collect();
            if !chosen.is_empty() {
                ctx.log(LogEvent::SkillTriggered {
                    skill: "ddddanggu",
                    owner,
                    presentation: None,
                });
                // LIFO stack: push in reverse so first chosen seat resolves Linked first!
                for &p in chosen.iter().rev() {
                    if ctx.core.player(p).alive && !ctx.core.player(p).linked {
                        ctx.emit(EventData::Linked {
                            player: p,
                            linked: true,
                        });
                    }
                }
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn step_dddlanghuai(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.stage {
        HostStage::Act => u01_wait_ask(m, ctx, HostStage::ShenmiguoRestage, 3057),
        HostStage::ShenmiguoRestage => {
            u01_clear_ask(ctx, owner);
            let forced = flag(ctx.core, owner, "dddxuanlun_del") != 0;
            let confirmed = matches!(ctx.answer().first(), Some(ChoiceOption::Confirm));
            if !confirmed && !forced {
                return Flow::Done;
            }
            ctx.core
                .vm_aux
                .flags
                .remove(&(owner, "dddxuanlun_del".to_string()));
            if let Some(root) = ctx
                .res
                .stack
                .iter()
                .rev()
                .find_map(|f| match &f.data {
                    EventData::DrawWindow { root, .. } => Some(*root),
                    EventData::DrawPhase { player, .. } if *player == owner => Some(f.id),
                    _ => None,
                })
            {
                ctx.cancel_draw_phase(root);
            }
            let hand = ctx.core.player(owner).hand.clone();
            if !hand.is_empty() {
                ctx.log(LogEvent::CardsRevealed {
                    cards: hand.clone(),
                    to: ctx.core.alive_players().collect(),
                });
            }
            let mut suits = Vec::new();
            for &c in &hand {
                let s = ctx.catalog.effective_card_suit(ctx.core, owner, c);
                if !suits.contains(&s) {
                    suits.push(s);
                }
            }
            let suit_cnt = suits.len().min(4) as u32;
            let state_flag = flag(ctx.core, owner, "dddlanghuai_state");
            let draw_num = if state_flag == 0 {
                suit_cnt
            } else {
                4u32.saturating_sub(suit_cnt)
            };
            set_flag(ctx.core, owner, "dddlanghuai_state", state_flag ^ 1);
            ctx.log(LogEvent::SkillTriggered {
                skill: "dddlanghuai",
                owner,
                presentation: None,
            });
            if draw_num > 0 {
                ctx.start_draw(owner, draw_num);
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn step_dddxuanlun(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    if m.card == Some(0) {
        match m.stage {
            HostStage::Act => u01_wait_ask(m, ctx, HostStage::ShenmiguoRestage, 3058),
            HostStage::ShenmiguoRestage => {
                u01_clear_ask(ctx, owner);
                if matches!(ctx.answer().first(), Some(ChoiceOption::Confirm)) {
                    set_flag(ctx.core, owner, "dddxuanlun_choose", 1);
                    set_flag(ctx.core, owner, "dddxuanlun_triggered", 1);
                    ctx.log(LogEvent::SkillTriggered {
                        skill: "dddxuanlun",
                        owner,
                        presentation: None,
                    });
                    ctx.start_draw(owner, 4);
                }
                Flow::Done
            }
            _ => Flow::Done,
        }
    } else {
        match m.stage {
            HostStage::Act => {
                ctx.core
                    .vm_aux
                    .flags
                    .remove(&(owner, "dddxuanlun_choose".to_string()));
                let del = flag(ctx.core, owner, "dddxuanlun_del") != 0;
                if del {
                    if he_cards(ctx.core, owner).is_empty() {
                        Flow::Done
                    } else {
                        u01_wait_ask(m, ctx, HostStage::SelectCard, 3060)
                    }
                } else {
                    u01_wait_ask(m, ctx, HostStage::SelectNature, 3059)
                }
            }
            HostStage::SelectNature => {
                u01_clear_ask(ctx, owner);
                match ctx.answer().first().copied() {
                    Some(ChoiceOption::Tag(1)) if flag(ctx.core, owner, "dddxuanlun_del") == 0 => {
                        set_flag(ctx.core, owner, "dddxuanlun_del", 1);
                        ctx.log(LogEvent::SkillTriggered {
                            skill: "dddxuanlun",
                            owner,
                            presentation: None,
                        });
                        Flow::Done
                    }
                    _ => {
                        if he_cards(ctx.core, owner).is_empty() {
                            Flow::Done
                        } else {
                            u01_wait_ask(m, ctx, HostStage::SelectCard, 3060)
                        }
                    }
                }
            }
            HostStage::SelectCard => {
                u01_clear_ask(ctx, owner);
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
                for &c in &picks {
                    u01_remove_owned_card_quiet(ctx, owner, c);
                }
                ctx.note_cards_lost(owner, &picks);
                ctx.core
                    .vm_aux
                    .piles
                    .insert((owner, "u01_xuanlun_hold".to_string()), picks);
                ctx.core
                    .vm_aux
                    .piles
                    .remove(&(owner, "u01_xuanlun_top".to_string()));
                u01_wait_ask(m, ctx, HostStage::ShuchuiTargetLoop, 3061)
            }
            HostStage::ShuchuiTargetLoop => {
                u01_clear_ask(ctx, owner);
                let top_picks: Vec<CardId> = ctx
                    .answer()
                    .iter()
                    .filter_map(|o| match o {
                        ChoiceOption::Card(c) => Some(*c),
                        _ => None,
                    })
                    .collect();
                let mut hold = ctx
                    .core
                    .vm_aux
                    .piles
                    .remove(&(owner, "u01_xuanlun_hold".to_string()))
                    .unwrap_or_default();
                let mut top = Vec::new();
                for c in top_picks {
                    if let Some(pos) = hold.iter().position(|&x| x == c) {
                        hold.remove(pos);
                        top.push(c);
                    }
                }
                ctx.core
                    .vm_aux
                    .piles
                    .insert((owner, "u01_xuanlun_top".to_string()), top);
                if hold.len() <= 1 {
                    finish_xuanlun_deck(ctx, owner, hold)
                } else {
                    ctx.core
                        .vm_aux
                        .piles
                        .insert((owner, "u01_xuanlun_hold".to_string()), hold);
                    u01_wait_ask(m, ctx, HostStage::CompareTargetPick, 3062)
                }
            }
            HostStage::CompareTargetPick => {
                u01_clear_ask(ctx, owner);
                let bot_picks: Vec<CardId> = ctx
                    .answer()
                    .iter()
                    .filter_map(|o| match o {
                        ChoiceOption::Card(c) => Some(*c),
                        _ => None,
                    })
                    .collect();
                let mut hold = ctx
                    .core
                    .vm_aux
                    .piles
                    .remove(&(owner, "u01_xuanlun_hold".to_string()))
                    .unwrap_or_default();
                let mut bottom = Vec::new();
                for c in bot_picks {
                    if let Some(pos) = hold.iter().position(|&x| x == c) {
                        hold.remove(pos);
                        bottom.push(c);
                    }
                }
                bottom.extend(hold);
                finish_xuanlun_deck(ctx, owner, bottom)
            }
            _ => Flow::Done,
        }
    }
}

fn finish_xuanlun_deck(ctx: &mut Ctx<'_>, owner: PlayerId, bottom: Vec<CardId>) -> Flow {
    let top = ctx
        .core
        .vm_aux
        .piles
        .remove(&(owner, "u01_xuanlun_top".to_string()))
        .unwrap_or_default();
    for c in top.into_iter().rev() {
        ctx.core.deck.push(c);
    }
    for c in bottom {
        ctx.core.deck.insert(0, c);
    }
    ctx.log(LogEvent::SkillTriggered {
        skill: "dddxuanlun",
        owner,
        presentation: None,
    });
    Flow::Done
}

fn step_dddjiexing(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    let is_reset = ctx
        .res
        .stack
        .last()
        .is_some_and(|f| matches!(f.stage, crate::event::Stage::AfterTriggers));
    if is_reset {
        if let Some(def) = ctx.catalog.skill_by_name("dddjiexing") {
            if let Some(inst) = ctx.core.skills.get_mut(&(owner, def)) {
                inst.once_used = false;
            }
        }
        set_flag(ctx.core, owner, "dddjiexing_used", 0);
        return Flow::Done;
    }
    match m.stage {
        HostStage::Act => u01_wait_ask(m, ctx, HostStage::ShenmiguoRestage, 3063),
        HostStage::ShenmiguoRestage => {
            u01_clear_ask(ctx, owner);
            if !matches!(ctx.answer().first(), Some(ChoiceOption::Confirm)) {
                return Flow::Done;
            }
            if let Some(def) = ctx.catalog.skill_by_name("dddjiexing") {
                if let Some(inst) = ctx.core.skills.get_mut(&(owner, def)) {
                    inst.once_used = true;
                }
            }
            set_flag(ctx.core, owner, "dddjiexing_used", 1);
            ctx.log(LogEvent::SkillTriggered {
                skill: "dddjiexing",
                owner,
                presentation: None,
            });
            if m.card == Some(1) {
                let rec = ctx.res.stack.iter().rev().find_map(|f| {
                    if let EventData::Recover { amount, .. } = &f.data {
                        Some((f.id, *amount))
                    } else {
                        None
                    }
                });
                if let Some((fid, amt)) = rec {
                    set_flag(ctx.core, 0, &format!("u01_recover_cancelled:{fid}"), 1);
                    if ctx.recover(owner, amt, None) > 0 {
                        if let Some(new_fid) = ctx.res.stack.last().map(|f| f.id) {
                            set_flag(
                                ctx.core,
                                owner,
                                &format!("u01_jiexing_self_recover:{new_fid}"),
                                1,
                            );
                        }
                    }
                }
            } else {
                let eq = ctx.res.stack.iter().rev().find_map(|f| {
                    if let EventData::Equipped { player, card, .. } = &f.data {
                        Some((f.id, *player, *card))
                    } else {
                        None
                    }
                });
                if let Some((fid, orig_player, eq_card)) = eq {
                    set_flag(ctx.core, orig_player, &format!("u01_equip_cancelled:{fid}"), 1);
                    if ctx.core.player(orig_player).hand.contains(&eq_card) {
                        ctx.take_hand_card(orig_player, eq_card);
                        ctx.core.table.push(eq_card);
                    }
                    ctx.emit(EventData::Equipped {
                        player: owner,
                        card: eq_card,
                        replaced: None,
                    });
                    if let Some(new_fid) = ctx.res.stack.last().map(|f| f.id) {
                        set_flag(ctx.core, owner, &format!("u01_equip_from_table:{new_fid}"), 1);
                        set_flag(ctx.core, owner, &format!("u01_jiexing_self_equip:{new_fid}"), 1);
                    }
                }
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn step_dddbailei(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.stage {
        HostStage::Act => u01_wait_ask(m, ctx, HostStage::TargetChoice, 3064),
        HostStage::TargetChoice => {
            u01_clear_ask(ctx, owner);
            let Some(ChoiceOption::Player(t)) = ctx.answer().first().copied() else {
                return Flow::Done;
            };
            m.target = Some(t);
            let is_max_eq =
                is_strict_max_equip(ctx.core, t) && !he_cards(ctx.core, t).is_empty();
            if is_max_eq {
                u01_wait_ask(m, ctx, HostStage::SelectCard, 3065)
            } else {
                dddbailei_deal_damage(m, ctx, t)
            }
        }
        HostStage::SelectCard => {
            u01_clear_ask(ctx, owner);
            let Some(t) = m.target else {
                return Flow::Done;
            };
            if let Some(ChoiceOption::Card(c)) = ctx.answer().first().copied() {
                u01_give_card_ex(ctx, t, owner, c, false);
                ctx.log(LogEvent::SkillTriggered {
                    skill: "dddbailei",
                    owner,
                    presentation: None,
                });
                Flow::Done
            } else if ctx.core.player(t).hp == 1 {
                dddbailei_deal_damage(m, ctx, t)
            } else {
                Flow::Done
            }
        }
        HostStage::ShuchuiTarget => {
            let Some(t) = m.target else {
                return Flow::Done;
            };
            if !ctx.core.player(t).alive {
                set_flag(ctx.core, owner, "dddbailei_lost", 1);
                if let Some(def) = ctx.catalog.skill_by_name("dddbailei") {
                    ctx.remove_skill(owner, def);
                }
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn dddbailei_deal_damage(m: &mut HostMachine, ctx: &mut Ctx<'_>, t: PlayerId) -> Flow {
    let owner = m.owner;
    ctx.log(LogEvent::SkillTriggered {
        skill: "dddbailei",
        owner,
        presentation: None,
    });
    ctx.emit(EventData::Damage(damage_data_with(
        Some(owner),
        t,
        1,
        DamageNature::Normal,
        None,
    )));
    m.stage = HostStage::ShuchuiTarget;
    Flow::Yield
}
