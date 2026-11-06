//! PA12 批⑬：确定性原语（122..=128）与交互式原语（109..=121）无 `vm_commit` 钩子时的确定性回退落账。

use super::commit::discard_some;
use super::*;

pub(super) fn commit_batch13_effect(
    ctx: &mut Ctx<'_>,
    executing: &str,
    owner: PlayerId,
    seats: u8,
    e: EffectRequest,
) {
    match e {
        EffectRequest::RecordCurrentShaAllTargets {
            player,
            card,
            skill,
        } => {
            let Some(p) = seat_of(player, seats) else {
                return;
            };
            if !ctx.core.player(p).alive {
                return;
            }
            let cid = card.raw();
            let sraw = skill.raw();
            let mut targets = Vec::new();
            for frame in ctx.res.stack.iter().rev() {
                if let EventData::CardUsed(u) = &frame.data {
                    if u.player == p
                        && (cid == 0 || u.card == Some(cid))
                        && matches!(
                            u.use_as.or_else(|| {
                                u.card.map(|c| ctx.catalog.cards[c as usize].kind)
                            }),
                            Some(CardKind::Sha | CardKind::FireSha | CardKind::LeiSha)
                        )
                    {
                        targets = u.targets.clone();
                        break;
                    }
                }
            }
            for t in &targets {
                ctx.core
                    .vm_aux
                    .flags
                    .insert((p, format!("sha_skill_target:{sraw}:{t}")), 1);
                ctx.core
                    .vm_aux
                    .flags
                    .insert((p, format!("sha_card_target:{cid}:{t}")), 1);
            }
            ctx.core.vm_aux.flags.insert(
                (p, format!("sha_all_targets_recorded:{sraw}")),
                targets.len() as u32,
            );
        }
        EffectRequest::TakeOneScopedCard { from, to, scope } => {
            let Some(b) = seat_of(to, seats) else {
                return;
            };
            let a = seat_of(from, seats).or_else(|| {
                ctx.res.stack.iter().rev().find_map(|f| match &f.data {
                    EventData::Damage(d) if d.target == b => d.source,
                    EventData::CardUsed(u) if u.targets.contains(&b) => Some(u.player),
                    _ => None,
                })
            });
            let Some(a) = a else {
                return;
            };
            if a == b || !ctx.core.player(a).alive || !ctx.core.player(b).alive {
                return;
            }
            take_first_scoped_card(ctx, a, b, scope);
        }
        EffectRequest::DrawAllEventTargets { players, count } => {
            if count == 0 {
                return;
            }
            let mut ordered: Vec<PlayerId> = players
                .into_iter()
                .filter_map(|p| seat_of(p, seats))
                .filter(|s| ctx.core.player(*s).alive)
                .collect();
            ordered.dedup();
            for p in ordered.into_iter().rev() {
                ctx.start_draw(p, count);
            }
        }
        EffectRequest::RefreshSkill { skill } => {
            let sraw = skill.raw();
            if let Some(def) = resolve_skill_def(ctx.catalog, sraw) {
                let name = ctx.catalog.skill_name(def).to_string();
                ctx.core
                    .vm_aux
                    .once
                    .retain(|(_, k, _)| k != &name && k != executing);
                for inst in ctx.core.skills.values_mut() {
                    inst.once_used = false;
                }
            } else {
                ctx.core.vm_aux.once.retain(|(_, k, _)| k != executing);
            }
        }
        EffectRequest::AddFirstMissingSkill {
            player, skill_defs, ..
        } => {
            let Some(p) = seat_of(player, seats) else {
                return;
            };
            if !ctx.core.player(p).alive {
                return;
            }
            for def_id in skill_defs {
                if !def_id.is_some() {
                    continue;
                }
                let raw = def_id.raw();
                if let Some(def) = resolve_skill_def(ctx.catalog, raw) {
                    if !ctx.core.has_skill(p, def) {
                        add_skill_instance(ctx, p, def);
                        return;
                    }
                } else {
                    let key = (p, format!("skill:{raw}"));
                    if let std::collections::btree_map::Entry::Vacant(e) =
                        ctx.core.vm_aux.flags.entry(key)
                    {
                        e.insert(1);
                        return;
                    }
                }
            }
        }
        EffectRequest::DiscardEquipmentSlot { player, slot } => {
            let Some(p) = seat_of(player, seats) else {
                return;
            };
            if !ctx.core.player(p).alive {
                return;
            }
            discard_equip_in_slot(ctx, p, slot);
        }
        EffectRequest::FinishCurrentPhaseAbortResolution { player, stage } => {
            let Some(p) = seat_of(player, seats) else {
                return;
            };
            if !ctx.core.player(p).alive {
                return;
            }
            ctx.core
                .vm_aux
                .flags
                .insert((p, "phase_aborted".to_string()), u32::from(stage).max(1));
            let phase = ctx.core.turn.phase;
            let mask = 1u32 << (phase as u8);
            *ctx.core
                .vm_aux
                .flags
                .entry((p, "skip_remaining_phases_mask".to_string()))
                .or_insert(0) |= mask;
            for frame in ctx.res.stack.iter_mut() {
                if let EventData::CardUsed(u) = &mut frame.data {
                    u.cancelled = true;
                }
            }
        }
        // 交互式原语（109..=121）在无 vm_commit 钩子时的确定性回退：
        EffectRequest::ChooseOtherDamage { player, amount, .. } => {
            let Some(p) = seat_of(player, seats) else {
                return;
            };
            if let Some(target) = first_other_alive(ctx.core, p) {
                let key = (p, "lifetime".to_string());
                *ctx.core.vm_aux.flags.entry(key).or_insert(0) += amount;
                ctx.emit(EventData::Damage(crate::machine::damage_data_with(
                    Some(p),
                    target,
                    amount as i32,
                    DamageNature::Normal,
                    None,
                )));
            }
        }
        EffectRequest::ChooseTargetDraw {
            player,
            count,
            scope,
            ..
        } => {
            let Some(p) = seat_of(player, seats) else {
                return;
            };
            let target = match scope {
                rule_api::DrawTargetScope::Other => first_other_alive(ctx.core, p),
                rule_api::DrawTargetScope::MaleOther => seats_from(ctx.core, p).into_iter().find(
                    |&q| {
                        q != p
                            && ctx.core.player(q).alive
                            && ctx.core.effective_gender(q)
                                == crate::vocabulary::generals::Gender::Male
                    },
                ),
                rule_api::DrawTargetScope::FemaleOther => {
                    seats_from(ctx.core, p).into_iter().find(|&q| {
                        q != p
                            && ctx.core.player(q).alive
                            && ctx.core.effective_gender(q)
                                == crate::vocabulary::generals::Gender::Female
                    })
                }
                rule_api::DrawTargetScope::HandLteChooser => {
                    let my_hand = ctx.core.player(p).hand.len();
                    seats_from(ctx.core, p).into_iter().find(|&q| {
                        q != p
                            && ctx.core.player(q).alive
                            && ctx.core.player(q).hand.len() <= my_hand
                    })
                }
                rule_api::DrawTargetScope::Any => ctx
                    .core
                    .player(p)
                    .alive
                    .then_some(p)
                    .or_else(|| first_other_alive(ctx.core, p)),
                rule_api::DrawTargetScope::MinHand => {
                    let min_h = ctx
                        .core
                        .players
                        .iter()
                        .filter(|ps| ps.alive)
                        .map(|ps| ps.hand.len())
                        .min();
                    min_h.and_then(|mh| {
                        seats_from(ctx.core, p)
                            .into_iter()
                            .find(|&q| ctx.core.player(q).alive && ctx.core.player(q).hand.len() == mh)
                    })
                }
                rule_api::DrawTargetScope::SameHp => {
                    let my_hp = ctx.core.player(p).hp;
                    seats_from(ctx.core, p)
                        .into_iter()
                        .find(|&q| ctx.core.player(q).alive && ctx.core.player(q).hp == my_hp)
                }
                rule_api::DrawTargetScope::HandEmpty => seats_from(ctx.core, p)
                    .into_iter()
                    .find(|&q| ctx.core.player(q).alive && ctx.core.player(q).hand.is_empty()),
                rule_api::DrawTargetScope::Wounded => seats_from(ctx.core, p)
                    .into_iter()
                    .find(|&q| ctx.core.player(q).alive && is_wounded(ctx.core, q)),
            };
            if let Some(t) = target {
                ctx.start_draw(t, count);
            }
        }
        EffectRequest::ChooseOtherRecover { player, amount } => {
            let Some(p) = seat_of(player, seats) else {
                return;
            };
            let target = seats_from(ctx.core, p).into_iter().find(|&q| {
                q != p && ctx.core.player(q).alive && is_wounded(ctx.core, q)
            });
            if let Some(t) = target {
                ctx.recover(t, amount, None);
            }
        }
        EffectRequest::ChooseDrawOrRecover {
            player,
            draw_count,
            recover_count,
            ..
        } => {
            let Some(p) = seat_of(player, seats) else {
                return;
            };
            if !ctx.core.player(p).alive {
                return;
            }
            if recover_count > 0 && is_wounded(ctx.core, p) && draw_count == 0 {
                ctx.recover(p, recover_count, None);
            } else if draw_count > 0 {
                ctx.start_draw(p, draw_count);
            } else if recover_count > 0 && is_wounded(ctx.core, p) {
                ctx.recover(p, recover_count, None);
            }
        }
        EffectRequest::ChooseOtherAddMark {
            player,
            mark,
            delta,
            ..
        } => {
            let Some(p) = seat_of(player, seats) else {
                return;
            };
            if let Some(t) = first_other_alive(ctx.core, p) {
                let e = ctx.core.vm_aux.marks.entry((t, mark)).or_insert(0);
                *e += delta;
                if *e <= 0 {
                    ctx.core.vm_aux.marks.remove(&(t, mark));
                }
            }
        }
        EffectRequest::ResolveHfjieying {
            player,
            target,
            grant_def,
            ..
        } => {
            let Some(p) = seat_of(player, seats) else {
                return;
            };
            let chosen = seat_of(target, seats)
                .filter(|&t| t != p && ctx.core.player(t).alive)
                .or_else(|| first_other_alive(ctx.core, p));
            if let Some(t) = chosen {
                let raw = grant_def.raw();
                if let Some(def) = resolve_skill_def(ctx.catalog, raw) {
                    add_skill_instance(ctx, t, def);
                } else {
                    ctx.core.vm_aux.flags.insert((t, format!("skill:{raw}")), 1);
                }
            }
        }
        EffectRequest::ChooseDiscardOneHandOrEquip { player } => {
            let Some(p) = seat_of(player, seats) else {
                return;
            };
            if let Some(c) = ctx
                .core
                .player(p)
                .hand
                .first()
                .copied()
                .or_else(|| ctx.core.player(p).equips.first().copied())
            {
                ctx.take_owned_card(p, c);
                discard_some(ctx, p, vec![c]);
            }
        }
        EffectRequest::DiscardBasicDrawEqual { player, .. } => {
            let Some(p) = seat_of(player, seats) else {
                return;
            };
            let basics: Vec<CardId> = ctx
                .core
                .player(p)
                .hand
                .iter()
                .copied()
                .filter(|&c| ctx.catalog.cards[c as usize].kind.is_basic_kind())
                .take(1)
                .collect();
            if !basics.is_empty() {
                for &c in &basics {
                    ctx.take_hand_card(p, c);
                }
                let k = basics.len() as u32;
                discard_some(ctx, p, basics);
                ctx.start_draw(p, k);
            }
        }
        EffectRequest::ChooseUseVirtualSha { player, .. } => {
            let Some(p) = seat_of(player, seats) else {
                return;
            };
            if let Some(t) = first_other_alive(ctx.core, p) {
                ctx.emit(EventData::CardUsed(crate::event::CardUseData {
                    card: None,
                    player: p,
                    targets: vec![t],
                    nature: DamageNature::Normal,
                    use_as: Some(CardKind::Sha),
                    cancelled: false,
                }));
            }
        }
        EffectRequest::ChooseOtherDiscardTargetCard { player, scope, .. } => {
            let Some(p) = seat_of(player, seats) else {
                return;
            };
            for q in seats_from(ctx.core, p) {
                if q != p && ctx.core.player(q).alive && discard_first_scoped_card(ctx, q, scope) {
                    break;
                }
            }
        }
        EffectRequest::ChooseDiscardTargetCard { target, scope, .. } => {
            let Some(t) = seat_of(target, seats) else {
                return;
            };
            if ctx.core.player(t).alive {
                let _ = discard_first_scoped_card(ctx, t, scope);
            }
        }
        EffectRequest::ChooseOtherGiveOneHandEquip { player, .. } => {
            let Some(p) = seat_of(player, seats) else {
                return;
            };
            let Some(t) = first_other_alive(ctx.core, p) else {
                return;
            };
            if let Some(c) = ctx.core.player(p).hand.first().copied() {
                ctx.give_hand_card(p, t, c);
            } else if let Some(c) = ctx.core.player(p).equips.first().copied() {
                ctx.take_owned_card(p, c);
                ctx.core.player_mut(t).hand.push(c);
                ctx.emit(EventData::CardsMoved {
                    cards: vec![c],
                    from: p,
                    to: t,
                });
            }
        }
        EffectRequest::ResolveDcdichou { .. } => {
            let _ = owner;
        }
        _ => super::commit_batch14::commit_batch14_effect(ctx, executing, owner, seats, e),
    }
}

fn seats_from(core: &GameStateCore, start: PlayerId) -> Vec<PlayerId> {
    let n = core.players.len();
    (0..n).map(|i| ((start as usize + i) % n) as u8).collect()
}

fn is_wounded(core: &GameStateCore, p: PlayerId) -> bool {
    let ps = core.player(p);
    ps.hp < ps.max_hp as i32
}

fn first_other_alive(core: &GameStateCore, p: PlayerId) -> Option<PlayerId> {
    seats_from(core, p)
        .into_iter()
        .find(|&q| q != p && core.player(q).alive)
}

fn add_skill_instance(ctx: &mut Ctx<'_>, p: PlayerId, def: crate::ids::SkillDefId) {
    ctx.ensure_skill(
        (p, def),
        SkillInstance {
            owner: p,
            enabled: true,
            once_used: false,
            expires_end_of_turn: false,
            rounds: 0,
        },
    );
}

fn discard_equip_in_slot(ctx: &mut Ctx<'_>, p: PlayerId, slot: u8) {
    let want = match slot {
        1 => crate::catalog::EquipSlot::Weapon,
        2 => crate::catalog::EquipSlot::Armor,
        3 => crate::catalog::EquipSlot::HorsePlus,
        4 => crate::catalog::EquipSlot::HorseMinus,
        5 => crate::catalog::EquipSlot::Treasure,
        _ => return,
    };
    let matching: Vec<CardId> = ctx
        .core
        .player(p)
        .equips
        .iter()
        .copied()
        .filter(|&c| ctx.catalog.card_slot_of(ctx.core, c) == Some(want))
        .collect();
    for &c in &matching {
        ctx.take_owned_card(p, c);
    }
    discard_some(ctx, p, matching);
}

fn take_first_scoped_card(
    ctx: &mut Ctx<'_>,
    from: PlayerId,
    to: PlayerId,
    scope: rule_api::TargetCardScope,
) {
    if scope.includes_hand() {
        if let Some(c) = ctx.core.player(from).hand.first().copied() {
            ctx.give_hand_card(from, to, c);
            return;
        }
    }
    if scope.includes_equip() {
        if let Some(c) = ctx.core.player(from).equips.first().copied() {
            ctx.take_owned_card(from, c);
            ctx.core.player_mut(to).hand.push(c);
            ctx.out.push(LogEvent::CardsTaken {
                from,
                to,
                cards: vec![c],
                public: true,
            });
            ctx.emit(EventData::CardsMoved {
                cards: vec![c],
                from,
                to,
            });
            return;
        }
    }
    if scope.includes_judge() {
        if let Some(c) = ctx.core.player(from).judge.first().copied() {
            ctx.take_owned_card(from, c);
            ctx.core.player_mut(to).hand.push(c);
            ctx.out.push(LogEvent::CardsTaken {
                from,
                to,
                cards: vec![c],
                public: true,
            });
            ctx.emit(EventData::CardsMoved {
                cards: vec![c],
                from,
                to,
            });
        }
    }
}

fn discard_first_scoped_card(
    ctx: &mut Ctx<'_>,
    target: PlayerId,
    scope: rule_api::TargetCardScope,
) -> bool {
    let card = (scope
        .includes_hand()
        .then(|| ctx.core.player(target).hand.first().copied())
        .flatten())
    .or_else(|| {
        scope
            .includes_equip()
            .then(|| ctx.core.player(target).equips.first().copied())
            .flatten()
    })
    .or_else(|| {
        scope
            .includes_judge()
            .then(|| ctx.core.player(target).judge.first().copied())
            .flatten()
    });
    if let Some(c) = card {
        ctx.take_owned_card(target, c);
        discard_some(ctx, target, vec![c]);
        true
    } else {
        false
    }
}
