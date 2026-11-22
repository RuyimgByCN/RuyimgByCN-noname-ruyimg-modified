//! Step handlers for Batch B part 2:
//! `dddshijian`, `dddxuyu_old`, `dddshijian_old`, `dddtaisi`, `dddquche`.

use super::*;

pub(super) fn step_b2(machine: &mut HostMachine, ctx: &mut Ctx<'_>) -> Option<Flow> {
    match machine.def_key.as_str() {
        "dddshijian" => Some(step_dddshijian(machine, ctx)),
        "dddxuyu_old" if machine.target.is_some() => {
            Some(step_dddxuyu_old_discard(machine, ctx))
        }
        "dddxuyu_old" => Some(step_dddxuyu_old(machine, ctx)),
        "dddshijian_old" => Some(step_dddshijian_old(machine, ctx)),
        "dddtaisi" => Some(step_dddtaisi(machine, ctx)),
        "dddquche" => Some(step_dddquche(machine, ctx)),
        _ => None,
    }
}

fn step_dddshijian(machine: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = machine.owner;
    if !ctx.core.player(owner).alive {
        return Flow::Done;
    }
    let is_mode_choice = machine.card.unwrap_or(0) >= 2;
    if !is_mode_choice {
        let Some(target) = machine.target else {
            return Flow::Done;
        };
        match machine.stage {
            HostStage::Act => {
                if !ctx.core.player(target).alive
                    || ctx.catalog.is_tieji_sealed(ctx.core, owner)
                    || he_cards(ctx.core, owner).is_empty()
                {
                    return Flow::Done;
                }
                wait_ask_2(machine, ctx, HostStage::SelectCard, 2018)
            }
            HostStage::SelectCard => {
                let Some(ChoiceOption::Card(c)) = ctx.answer().first().copied() else {
                    return Flow::Done;
                };
                if !ctx.core.player(target).alive {
                    return Flow::Done;
                }
                ctx.log(LogEvent::SkillTriggered {
                    skill: "dddshijian",
                    owner,
                    presentation: None,
                });
                u01_give_card_ex(ctx, owner, target, c, false);
                let seq = flag(ctx.core, 0, "play_phase_seq").max(1);
                set_flag(ctx.core, owner, "dddshijian_extra_target", u32::from(target) + 1);
                set_flag(ctx.core, owner, "dddshijian_extra_seq", seq);
                Flow::Done
            }
            _ => Flow::Done,
        }
    } else {
        match machine.stage {
            HostStage::Act => wait_ask_2(machine, ctx, HostStage::TargetChoice, 2019),
            HostStage::TargetChoice => {
                let Some(ChoiceOption::Tag(tag)) = ctx.answer().first().copied() else {
                    return Flow::Done;
                };
                ctx.core
                    .vm_aux
                    .flags
                    .remove(&(owner, "dddshijian_extra_target".to_string()));
                ctx.core
                    .vm_aux
                    .flags
                    .remove(&(owner, "dddshijian_extra_seq".to_string()));
                ctx.log(LogEvent::SkillTriggered {
                    skill: "dddshijian",
                    owner,
                    presentation: None,
                });
                if tag == 0 {
                    if let Some(use_frame) = ctx
                        .res
                        .stack
                        .iter()
                        .rev()
                        .find(|f| matches!(f.data, EventData::CardUsed(_)))
                    {
                        let fid = use_frame.id;
                        let cur = flag(ctx.core, ctx.core.turn.current, &format!("card_use_extra_effect:{fid}"));
                        set_flag(
                            ctx.core,
                            ctx.core.turn.current,
                            &format!("card_use_extra_effect:{fid}"),
                            cur + 1,
                        );
                    }
                } else {
                    ctx.start_draw(owner, 1);
                }
                Flow::Done
            }
            _ => Flow::Done,
        }
    }
}

fn step_dddxuyu_old(machine: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = machine.owner;
    if !ctx.core.player(owner).alive || ctx.catalog.is_tieji_sealed(ctx.core, owner) {
        return Flow::Done;
    }
    match machine.stage {
        HostStage::Act => wait_ask_2(machine, ctx, HostStage::TargetChoice, 2020),
        HostStage::TargetChoice => {
            let seq = ctx
                .core
                .vm_aux
                .flags
                .remove(&(owner, "dddxuyu_old_play_seq".to_string()))
                .unwrap_or(1)
                .max(1);
            if !matches!(ctx.answer().first(), Some(ChoiceOption::Confirm)) {
                return Flow::Done;
            }
            ctx.log(LogEvent::SkillTriggered {
                skill: "dddxuyu_old",
                owner,
                presentation: None,
            });
            set_flag(ctx.core, owner, &format!("dddxuyu_old_armed:{seq}"), 1);
            ctx.start_draw(owner, 2);
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn step_dddxuyu_old_discard(machine: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = machine.owner;
    if !ctx.core.player(owner).alive {
        return Flow::Done;
    }
    match machine.stage {
        HostStage::Act => {
            if he_cards(ctx.core, owner).is_empty() {
                return Flow::Done;
            }
            wait_ask_2(machine, ctx, HostStage::SelectCard, 2021)
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

fn step_dddshijian_old(machine: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = machine.owner;
    let Some(target) = machine.target else {
        return Flow::Done;
    };
    if !ctx.core.player(owner).alive
        || !ctx.core.player(target).alive
        || ctx.catalog.is_tieji_sealed(ctx.core, owner)
    {
        return Flow::Done;
    }
    match machine.stage {
        HostStage::Act => wait_ask_2(machine, ctx, HostStage::TargetChoice, 2022),
        HostStage::TargetChoice => {
            if !matches!(ctx.answer().first(), Some(ChoiceOption::Confirm)) {
                return Flow::Done;
            }
            let seq = machine.card.unwrap_or(1);
            ctx.log(LogEvent::SkillTriggered {
                skill: "dddshijian_old",
                owner,
                presentation: None,
            });
            set_flag(
                ctx.core,
                owner,
                &format!("dddshijian_old_armed:{seq}"),
                u32::from(target) + 1,
            );
            ctx.start_draw(target, 1);
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn step_dddtaisi(machine: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = machine.owner;
    if !ctx.core.player(owner).alive || ctx.catalog.is_tieji_sealed(ctx.core, owner) {
        return Flow::Done;
    }
    match machine.stage {
        HostStage::Act => {
            if flag(ctx.core, owner, "turn_change_hp") == 0
                || dddtaisi_discard_cards(ctx.core).is_empty()
            {
                return Flow::Done;
            }
            wait_ask_2(machine, ctx, HostStage::TargetChoice, 2023)
        }
        HostStage::TargetChoice => {
            let Some(ChoiceOption::Player(t)) = ctx.answer().first().copied() else {
                return Flow::Done;
            };
            if dddtaisi_discard_cards(ctx.core).is_empty() {
                return Flow::Done;
            }
            machine.target = Some(t);
            wait_ask_2(machine, ctx, HostStage::SelectCard, 2024)
        }
        HostStage::SelectCard => {
            let Some(target) = machine.target else {
                return Flow::Done;
            };
            let Some(ChoiceOption::Card(c)) = ctx.answer().first().copied() else {
                return Flow::Done;
            };
            if !ctx.core.discard.contains(&c) || !ctx.core.player(target).alive {
                return Flow::Done;
            }
            ctx.log(LogEvent::SkillTriggered {
                skill: "dddtaisi",
                owner,
                presentation: None,
            });
            ctx.core.discard.retain(|&x| x != c);
            ctx.core.player_mut(target).hand.push(c);
            ctx.log(LogEvent::CardsGivenPublic {
                from: owner,
                to: target,
                cards: vec![c],
            });
            let mask = flag(ctx.core, owner, "turn_dmg_sources_mask");
            if (mask & (1u32 << target)) != 0 {
                ctx.start_draw(owner, 2);
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn step_dddquche(machine: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = machine.owner;
    if !ctx.core.player(owner).alive {
        return Flow::Done;
    }
    match machine.stage {
        HostStage::Act => {
            if is_scope4_used(ctx.core, owner, "dddquche")
                || dddquche_red_cards(ctx.core, ctx.catalog, owner).is_empty()
                || !ctx.core.alive_players().any(|p| p != owner)
            {
                return Flow::Done;
            }
            wait_ask_2(machine, ctx, HostStage::SelectCard, 2025)
        }
        HostStage::SelectCard => {
            let Some(ChoiceOption::Card(red)) = ctx.answer().first().copied() else {
                return Flow::Done;
            };
            machine.card = Some(red);
            wait_ask_2(machine, ctx, HostStage::TargetChoice, 2026)
        }
        HostStage::TargetChoice => {
            let Some(red) = machine.card else {
                return Flow::Done;
            };
            let Some(ChoiceOption::Player(t)) = ctx.answer().first().copied() else {
                return Flow::Done;
            };
            if !ctx.core.player(t).alive {
                return Flow::Done;
            }
            if !is_scope4_used(ctx.core, owner, "dddquche") {
                mark_scope4_used(ctx.core, owner, "dddquche");
                if let Some(def) = ctx.catalog.skill_by_name("dddquche") {
                    if let Some(inst) = ctx.core.skills.get_mut(&(owner, def)) {
                        inst.once_used = true;
                    }
                }
            }
            ctx.log(LogEvent::SkillTriggered {
                skill: "dddquche",
                owner,
                presentation: None,
            });
            u01_give_card_ex(ctx, owner, t, red, false);
            let marks = flag(ctx.core, owner, "dddquche_effect_marks");
            set_flag(ctx.core, owner, "dddquche_effect_marks", marks + 1);
            machine.card = None;
            if dddquche_black_cards(ctx.core, ctx.catalog, owner).is_empty()
                || virtual_legal_targets(ctx.core, ctx.catalog, owner, CardKind::Sha).is_empty()
            {
                return Flow::Done;
            }
            wait_ask_2(machine, ctx, HostStage::CardsAsk, 2027)
        }
        HostStage::CardsAsk => {
            let Some(ChoiceOption::Card(blk)) = ctx.answer().first().copied() else {
                return Flow::Done;
            };
            machine.card = Some(blk);
            wait_ask_2(machine, ctx, HostStage::SelectNature, 2028)
        }
        HostStage::SelectNature => {
            let Some(blk) = machine.card else {
                return Flow::Done;
            };
            let Some(ChoiceOption::Player(sha_target)) = ctx.answer().first().copied() else {
                return Flow::Done;
            };
            if u01_remove_owned_ex(ctx, owner, blk).is_none() {
                return Flow::Done;
            }
            ctx.core.table.push(blk);
            set_flag(ctx.core, owner, "vm_va_sha_no_count", 1);
            let marks = flag(ctx.core, owner, "dddquche_effect_marks");
            set_flag(ctx.core, owner, "dddquche_effect_marks", marks + 1);
            machine.card = None;
            machine.stage = HostStage::DrawSettle;
            ctx.emit(EventData::CardUsed(CardUseData {
                card: Some(blk),
                player: owner,
                targets: vec![sha_target],
                nature: DamageNature::Normal,
                use_as: Some(CardKind::Sha),
                cancelled: false,
            }));
            Flow::Yield
        }
        HostStage::DrawSettle => {
            if dddquche_red_cards(ctx.core, ctx.catalog, owner).is_empty()
                || !ctx.core.alive_players().any(|p| p != owner)
            {
                return Flow::Done;
            }
            wait_ask_2(machine, ctx, HostStage::SelectCard, 2025)
        }
        _ => Flow::Done,
    }
}
