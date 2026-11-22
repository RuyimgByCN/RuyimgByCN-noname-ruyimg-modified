//! Step handlers for Batch C part 2:
//! `dddfusi_global`, `dddtuoji`, `dddchashi`, `dddmiaoxing`, `dddfenji`.

use super::*;

pub(super) fn step_c2(machine: &mut HostMachine, ctx: &mut Ctx<'_>) -> Option<Flow> {
    match machine.def_key.as_str() {
        "dddfusi_global" => Some(step_dddfusi_global(machine, ctx)),
        "dddtuoji" => Some(step_dddtuoji(machine, ctx)),
        "dddchashi" => Some(step_dddchashi(machine, ctx)),
        "dddmiaoxing" => Some(step_dddmiaoxing(machine, ctx)),
        "dddfenji" => Some(step_dddfenji(machine, ctx)),
        _ => None,
    }
}

fn finish_fusi_park(machine: &mut HostMachine, ctx: &mut Ctx<'_>, owner: PlayerId, ok: bool) -> Flow {
    let was_respond = flag(ctx.core, owner, "u01_fusi_need") != 0;
    ctx.core
        .vm_aux
        .flags
        .remove(&(owner, "u01_fusi_need".to_string()));
    ctx.core
        .vm_aux
        .flags
        .remove(&(owner, "u01_fusi_picked_card".to_string()));
    let _ = machine;
    if was_respond {
        finish_respond_park_pub(ctx, owner, ok)
    } else {
        Flow::Done
    }
}

fn commit_fusi_borrow(machine: &mut HostMachine, ctx: &mut Ctx<'_>, owner: PlayerId) -> Flow {
    let Some(src) = machine.target else {
        return finish_fusi_park(machine, ctx, owner, false);
    };
    let need_raw = flag(ctx.core, owner, "u01_fusi_need");
    let picked_card = if need_raw != 0 {
        flag(ctx.core, owner, "u01_fusi_picked_card")
    } else {
        machine.card.unwrap_or(0)
    };
    if !ctx.core.player(src).hand.contains(&picked_card)
        || !is_u01_shown_card(ctx.core, src, picked_card)
    {
        return finish_fusi_park(machine, ctx, owner, false);
    }
    ctx.log(LogEvent::SkillTriggered {
        skill: "dddfusi_global",
        owner,
        presentation: None,
    });
    ctx.take_hand_card(src, picked_card);
    ctx.note_cards_lost(src, &[picked_card]);
    ctx.emit(EventData::CardsLost {
        player: src,
        cards: vec![picked_card],
        origins: vec![LostOrigin::Hand],
    });
    if need_raw != 0 {
        let is_use = machine.targets.first().copied() == Some(1);
        if !is_use {
            return finish_fusi_park(machine, ctx, owner, false);
        }
        let need = decode_respond_need(need_raw);
        ctx.core.table.push(picked_card);
        set_flag(ctx.core, owner, "u01_respond_card_override", picked_card + 1);
        set_flag(ctx.core, owner, "u01_respond_fusi_owner", u32::from(src) + 1);
        if need == CardKind::Shan {
            set_flag(ctx.core, owner, "u01_respond_emit_use", 1);
        }
        return finish_fusi_park(machine, ctx, owner, true);
    }
    ctx.core
        .vm_aux
        .flags
        .remove(&(owner, "u01_fusi_need".to_string()));
    ctx.core
        .vm_aux
        .flags
        .remove(&(owner, "u01_fusi_picked_card".to_string()));
    ctx.core.table.push(picked_card);
    let kind = ctx.catalog.card_kind(picked_card);
    if kind.is_equip() {
        let new_fid = ctx.res.next_event_id;
        set_flag(
            ctx.core,
            owner,
            &format!("u01_equip_from_table:{new_fid}"),
            1,
        );
        set_flag(
            ctx.core,
            owner,
            &format!("dddfusi_used_owner_eq:{new_fid}"),
            u32::from(src) + 1,
        );
        if ctx.catalog.skill_by_name("dddchashi").is_some() {
            set_flag(ctx.core, owner, &format!("chashi_equip_use:{new_fid}"), 1);
        }
        ctx.emit(EventData::Equipped {
            player: owner,
            card: picked_card,
            replaced: None,
        });
        return Flow::Done;
    }
    let nature = match kind {
        CardKind::FireSha | CardKind::Linghunzhihuo => DamageNature::Fire,
        CardKind::LeiSha => DamageNature::Thunder,
        _ => DamageNature::Normal,
    };
    let targets = if dddlingyong_needs_target_prompt(ctx.catalog, picked_card)
        || kind == CardKind::Guiyoujie
    {
        machine.targets.clone()
    } else {
        dddfusi_play_targets(ctx.core, ctx.catalog, owner, picked_card).unwrap_or_default()
    };
    let new_fid = ctx.res.next_event_id;
    set_flag(
        ctx.core,
        owner,
        &format!("dddfusi_used_owner:{new_fid}"),
        u32::from(src) + 1,
    );
    ctx.emit(EventData::CardUsed(CardUseData {
        card: Some(picked_card),
        player: owner,
        targets,
        nature,
        use_as: None,
        cancelled: false,
    }));
    Flow::Done
}

fn step_dddfusi_global(machine: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = machine.owner;
    if !ctx.core.player(owner).alive {
        return finish_fusi_park(machine, ctx, owner, false);
    }
    match machine.stage {
        HostStage::Act => {
            if let Some(need_raw) = machine.card {
                set_flag(ctx.core, owner, "u01_fusi_need", need_raw);
            } else {
                ctx.core
                    .vm_aux
                    .flags
                    .remove(&(owner, "u01_fusi_need".to_string()));
            }
            wait_ask_2(machine, ctx, HostStage::SelectCard, 2040)
        }
        HostStage::SelectCard => {
            let Some(ChoiceOption::Card(c)) = ctx.answer().first().copied() else {
                return finish_fusi_park(machine, ctx, owner, false);
            };
            let Some(src) = find_shown_card_owner(ctx.core, ctx.catalog, owner, c) else {
                return finish_fusi_park(machine, ctx, owner, false);
            };
            machine.target = Some(src);
            if flag(ctx.core, owner, "u01_fusi_need") != 0 {
                set_flag(ctx.core, owner, "u01_fusi_picked_card", c);
                if flag(ctx.core, owner, "dddfusi_allowed") != 0 {
                    return commit_fusi_borrow(machine, ctx, owner);
                }
                return wait_ask_2(machine, ctx, HostStage::CompareTargetPick, 2042);
            }
            machine.card = Some(c);
            let kind = ctx.catalog.card_kind(c);
            if dddlingyong_needs_target_prompt(ctx.catalog, c) || kind == CardKind::Guiyoujie {
                return wait_ask_2(machine, ctx, HostStage::TargetChoice, 2041);
            }
            if flag(ctx.core, owner, "dddfusi_allowed") != 0 {
                return commit_fusi_borrow(machine, ctx, owner);
            }
            wait_ask_2(machine, ctx, HostStage::CompareTargetPick, 2042)
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
                return finish_fusi_park(machine, ctx, owner, false);
            }
            let cid = machine.card.unwrap_or(0);
            if ctx.catalog.card_kind(cid) == CardKind::Jiedao {
                let holder = targets[0];
                machine.targets = vec![holder];
                let old_target = machine.target.unwrap_or(owner);
                set_flag(ctx.core, owner, "u01_fusi_src_bak", u32::from(old_target));
                machine.target = Some(holder);
                return wait_ask_2(machine, ctx, HostStage::JiefenTarget, 2052);
            }
            machine.targets = targets;
            if flag(ctx.core, owner, "dddfusi_allowed") != 0 {
                return commit_fusi_borrow(machine, ctx, owner);
            }
            wait_ask_2(machine, ctx, HostStage::CompareTargetPick, 2042)
        }
        HostStage::JiefenTarget => {
            let Some(ChoiceOption::Player(victim)) = ctx.answer().first().copied() else {
                return finish_fusi_park(machine, ctx, owner, false);
            };
            let src = ctx
                .core
                .vm_aux
                .flags
                .remove(&(owner, "u01_fusi_src_bak".to_string()))
                .unwrap_or(0) as PlayerId;
            machine.target = Some(src);
            machine.targets.push(victim);
            if flag(ctx.core, owner, "dddfusi_allowed") != 0 {
                return commit_fusi_borrow(machine, ctx, owner);
            }
            wait_ask_2(machine, ctx, HostStage::CompareTargetPick, 2042)
        }
        HostStage::CompareTargetPick => {
            let Some(ChoiceOption::Tag(tag)) = ctx.answer().first().copied() else {
                return finish_fusi_park(machine, ctx, owner, false);
            };
            match tag {
                0 => commit_fusi_borrow(machine, ctx, owner),
                2 => {
                    set_flag(ctx.core, owner, "dddfusi_allowed", 1);
                    commit_fusi_borrow(machine, ctx, owner)
                }
                3 => {
                    set_flag(ctx.core, owner, "dddfusi_window_refused", 1);
                    set_flag(ctx.core, owner, "dddfusi_refused", 1);
                    finish_fusi_park(machine, ctx, owner, false)
                }
                _ => {
                    set_flag(ctx.core, owner, "dddfusi_window_refused", 1);
                    finish_fusi_park(machine, ctx, owner, false)
                }
            }
        }
        _ => finish_fusi_park(machine, ctx, owner, false),
    }
}

fn step_dddtuoji(machine: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = machine.owner;
    if !ctx.core.player(owner).alive || ctx.catalog.is_tieji_sealed(ctx.core, owner) {
        return Flow::Done;
    }
    match machine.stage {
        HostStage::Act => wait_ask_2(machine, ctx, HostStage::TargetChoice, 2043),
        HostStage::TargetChoice => {
            if matches!(ctx.answer().first(), Some(ChoiceOption::Confirm)) {
                ctx.log(LogEvent::SkillTriggered {
                    skill: "dddtuoji",
                    owner,
                    presentation: None,
                });
                ctx.start_draw(owner, 3);
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn step_dddchashi(machine: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = machine.owner;
    if !ctx.core.player(owner).alive || ctx.catalog.is_tieji_sealed(ctx.core, owner) {
        return Flow::Done;
    }
    if machine.card == Some(1) {
        let Some(target) = machine.target else {
            return Flow::Done;
        };
        if flag(ctx.core, owner, "dddchashi_armed") == 0 {
            return Flow::Done;
        }
        ctx.core
            .vm_aux
            .flags
            .remove(&(owner, "dddchashi_armed".to_string()));
        ctx.core
            .vm_aux
            .flags
            .remove(&(owner, "dddchashi_target".to_string()));
        ctx.core
            .vm_aux
            .flags
            .remove(&(owner, "dddchashi_suit".to_string()));
        ctx.core
            .vm_aux
            .flags
            .remove(&(owner, "dddchashi_kind".to_string()));
        ctx.log(LogEvent::SkillTriggered {
            skill: "dddchashi",
            owner,
            presentation: None,
        });
        ctx.start_draw(owner, 1);
        if ctx.core.player(target).alive {
            ctx.start_draw(target, 1);
        }
        return Flow::Done;
    }
    let Some(target) = machine.target else {
        return Flow::Done;
    };
    match machine.stage {
        HostStage::Act => {
            if he_cards(ctx.core, owner).is_empty() {
                return Flow::Done;
            }
            wait_ask_2(machine, ctx, HostStage::SelectCard, 2044)
        }
        HostStage::SelectCard => {
            let Some(ChoiceOption::Card(c)) = ctx.answer().first().copied() else {
                return Flow::Done;
            };
            let sbit = effective_suit_bit(ctx.core, ctx.catalog, owner, c);
            let kgrp = card_kind_group(ctx.catalog.card_kind(c));
            ctx.log(LogEvent::SkillTriggered {
                skill: "dddchashi",
                owner,
                presentation: None,
            });
            u01_discard_batch(ctx, owner, &[c]);
            set_flag(ctx.core, owner, "dddchashi_armed", 1);
            set_flag(ctx.core, owner, "dddchashi_target", u32::from(target) + 1);
            set_flag(ctx.core, owner, "dddchashi_suit", sbit);
            set_flag(ctx.core, owner, "dddchashi_kind", kgrp);
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn step_dddmiaoxing(machine: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = machine.owner;
    if !ctx.core.player(owner).alive {
        return Flow::Done;
    }
    let is_out = machine.card == Some(10) || flag(ctx.core, owner, "dddmiaoxing_init_done") == 0;
    if is_out {
        match machine.stage {
            HostStage::Act => {
                if ctx.core.player(owner).hand.len() < 3 {
                    return Flow::Done;
                }
                wait_ask_2(machine, ctx, HostStage::SelectNature, 2045)
            }
            HostStage::SelectNature => {
                let tag = match ctx.answer().first().copied() {
                    Some(ChoiceOption::Tag(t)) if t < 3 => t as usize,
                    _ => 0,
                };
                let hand = ctx.core.player(owner).hand.clone();
                if hand.len() < 3 {
                    return Flow::Done;
                }
                let groups = dddmiaoxing_split_3(&hand);
                let mut pile_groups = Vec::with_capacity(2);
                for (i, grp) in groups.iter().enumerate() {
                    if i != tag {
                        pile_groups.push(grp.clone());
                    }
                }
                ctx.log(LogEvent::SkillTriggered {
                    skill: "dddmiaoxing",
                    owner,
                    presentation: None,
                });
                ctx.core.player_mut(owner).hand = groups[tag].clone();
                ctx.core
                    .vm_aux
                    .piles
                    .insert((owner, "dddmiaoxing_1".to_string()), pile_groups[0].clone());
                ctx.core
                    .vm_aux
                    .piles
                    .insert((owner, "dddmiaoxing_2".to_string()), pile_groups[1].clone());
                set_flag(ctx.core, owner, "dddmiaoxing_init_done", 1);
                Flow::Done
            }
            _ => Flow::Done,
        }
    } else {
        match machine.stage {
            HostStage::Act => {
                if dddmiaoxing_mismatch_tags(ctx.core, owner).is_empty() {
                    return Flow::Done;
                }
                wait_ask_2(machine, ctx, HostStage::TargetChoice, 2046)
            }
            HostStage::TargetChoice => {
                let mut tags: Vec<u8> = ctx
                    .answer()
                    .iter()
                    .filter_map(|o| match o {
                        ChoiceOption::Tag(t) if *t < 2 => Some(*t),
                        _ => None,
                    })
                    .collect();
                tags.sort_unstable();
                tags.dedup();
                if tags.is_empty() {
                    return Flow::Done;
                }
                ctx.log(LogEvent::SkillTriggered {
                    skill: "dddmiaoxing",
                    owner,
                    presentation: None,
                });
                machine.targets = tags;
                process_miaoxing_balance_queue(machine, ctx, owner)
            }
            HostStage::SelectCard => {
                let tag = machine.card.unwrap_or(0);
                let key = if tag == 1 {
                    "dddmiaoxing_2"
                } else {
                    "dddmiaoxing_1"
                };
                let drops: Vec<CardId> = ctx
                    .answer()
                    .iter()
                    .filter_map(|o| match o {
                        ChoiceOption::Card(c) => Some(*c),
                        _ => None,
                    })
                    .collect();
                let mut dropped = Vec::new();
                if let Some(pile) = ctx.core.vm_aux.piles.get_mut(&(owner, key.to_string())) {
                    for &c in &drops {
                        if let Some(pos) = pile.iter().position(|&x| x == c) {
                            pile.remove(pos);
                            dropped.push(c);
                        }
                    }
                }
                for c in dropped {
                    ctx.to_discard(c);
                }
                process_miaoxing_balance_queue(machine, ctx, owner)
            }
            _ => Flow::Done,
        }
    }
}

fn process_miaoxing_balance_queue(
    machine: &mut HostMachine,
    ctx: &mut Ctx<'_>,
    owner: PlayerId,
) -> Flow {
    while !machine.targets.is_empty() {
        let tag = machine.targets.remove(0);
        let key = if tag == 1 {
            "dddmiaoxing_2"
        } else {
            "dddmiaoxing_1"
        };
        let hand_len = ctx.core.player(owner).hand.len();
        let pile_len = ctx
            .core
            .vm_aux
            .piles
            .get(&(owner, key.to_string()))
            .map_or(0, Vec::len);
        if pile_len == hand_len {
            continue;
        }
        if pile_len < hand_len {
            let need = hand_len - pile_len;
            let mut gained = Vec::with_capacity(need);
            for _ in 0..need {
                if let Some(c) = draw_into_deck_if_needed(ctx.core) {
                    gained.push(c);
                }
            }
            ctx.core
                .vm_aux
                .piles
                .entry((owner, key.to_string()))
                .or_default()
                .extend(gained);
        } else {
            machine.card = Some(u32::from(tag));
            return wait_ask_2(machine, ctx, HostStage::SelectCard, 2047);
        }
    }
    Flow::Done
}

fn step_dddfenji(machine: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = machine.owner;
    if !ctx.core.player(owner).alive || ctx.catalog.is_tieji_sealed(ctx.core, owner) {
        ctx.core
            .vm_aux
            .flags
            .remove(&(owner, "dddfenji_phase_mode".to_string()));
        return Flow::Done;
    }
    match machine.stage {
        HostStage::Act => {
            if let Some(mode) = machine.card {
                set_flag(ctx.core, owner, "dddfenji_phase_mode", mode);
            }
            wait_ask_2(machine, ctx, HostStage::SelectNature, 2048)
        }
        HostStage::SelectNature => {
            let Some(ChoiceOption::Tag(tag)) = ctx.answer().first().copied() else {
                ctx.core
                    .vm_aux
                    .flags
                    .remove(&(owner, "dddfenji_phase_mode".to_string()));
                return Flow::Done;
            };
            let Some(kind) = dddfenji_kind_for_tag(ctx.catalog, tag) else {
                return Flow::Done;
            };
            let mask = flag(ctx.core, owner, "dddfenji_used_mask") | (1u32 << tag);
            set_flag(ctx.core, owner, "dddfenji_used_mask", mask);
            let is_draw = flag(ctx.core, owner, "dddfenji_phase_mode") == 1;
            if is_draw {
                for frame in ctx.res.stack.iter_mut().rev() {
                    if let EventData::DrawPhase {
                        player,
                        pending,
                        num_fixed,
                        ..
                    } = &mut frame.data
                    {
                        if *player == owner {
                            *pending = 0;
                            *num_fixed = true;
                            break;
                        }
                    }
                }
            } else {
                ctx.core.player_mut(owner).skip_discard_phase = true;
            }
            ctx.log(LogEvent::SkillTriggered {
                skill: "dddfenji",
                owner,
                presentation: None,
            });
            machine.card = Some(u32::from(tag));
            if virtual_kind_needs_target_prompt(kind) {
                return wait_ask_2(machine, ctx, HostStage::TargetChoice, 2049);
            }
            let targets = dddfenji_legal_targets(ctx.core, ctx.catalog, owner, kind);
            emit_fenji_trick(machine, ctx, owner, kind, targets)
        }
        HostStage::TargetChoice => {
            let Some(tag) = machine.card.map(|v| v as u8) else {
                return Flow::Done;
            };
            let Some(kind) = dddfenji_kind_for_tag(ctx.catalog, tag) else {
                return Flow::Done;
            };
            let targets: Vec<PlayerId> = ctx
                .answer()
                .iter()
                .filter_map(|o| match o {
                    ChoiceOption::Player(p) => Some(*p),
                    _ => None,
                })
                .collect();
            if targets.is_empty() {
                return Flow::Done;
            }
            if kind == CardKind::Jiedao {
                machine.target = Some(targets[0]);
                return wait_ask_2(machine, ctx, HostStage::JiefenTarget, 2053);
            }
            emit_fenji_trick(machine, ctx, owner, kind, targets)
        }
        HostStage::JiefenTarget => {
            let Some(holder) = machine.target else {
                return Flow::Done;
            };
            let Some(ChoiceOption::Player(victim)) = ctx.answer().first().copied() else {
                return Flow::Done;
            };
            emit_fenji_trick(machine, ctx, owner, CardKind::Jiedao, vec![holder, victim])
        }
        HostStage::DrawSettle => {
            let dmg = ctx
                .core
                .vm_aux
                .flags
                .remove(&(owner, "dddfenji_result_dmg".to_string()))
                .unwrap_or(1)
                .saturating_sub(1);
            let target_cnt = ctx
                .core
                .vm_aux
                .flags
                .remove(&(owner, "dddfenji_result_targets".to_string()))
                .unwrap_or(1)
                .saturating_sub(1);
            let is_draw = ctx
                .core
                .vm_aux
                .flags
                .remove(&(owner, "dddfenji_phase_mode".to_string()))
                .unwrap_or(1)
                == 1;
            if is_draw {
                if dmg > 0 {
                    ctx.start_draw(owner, dmg);
                }
                return Flow::Done;
            }
            let n = (target_cnt as usize).min(ctx.core.player(owner).hand.len());
            if n == 0 {
                return Flow::Done;
            }
            machine.card = Some(n as u32);
            wait_ask_2(machine, ctx, HostStage::SelectCard, 2050)
        }
        HostStage::SelectCard => {
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
}

fn emit_fenji_trick(
    machine: &mut HostMachine,
    ctx: &mut Ctx<'_>,
    owner: PlayerId,
    kind: CardKind,
    targets: Vec<PlayerId>,
) -> Flow {
    let nature = match kind {
        CardKind::Linghunzhihuo | CardKind::FireSha => DamageNature::Fire,
        CardKind::LeiSha => DamageNature::Thunder,
        _ => DamageNature::Normal,
    };
    let next_fid = ctx.res.next_event_id;
    set_flag(ctx.core, owner, "dddfenji_active_fid", next_fid);
    machine.stage = HostStage::DrawSettle;
    ctx.emit(EventData::CardUsed(CardUseData {
        card: None,
        player: owner,
        targets,
        nature,
        use_as: Some(kind),
        cancelled: false,
    }));
    Flow::Yield
}
