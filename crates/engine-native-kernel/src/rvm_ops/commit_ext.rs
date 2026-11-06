//! PA12 批⑫：25 种选牌/选目标与确定性效果落账（84..=108）。

use super::commit::{discard_some, pop_deck, push_pile};
use super::*;

pub(super) fn commit_ext_effect(
    ctx: &mut Ctx<'_>,
    executing: &str,
    _owner: PlayerId,
    seats: u8,
    e: EffectRequest,
) {
    match e {
        EffectRequest::DiscardCards { player, cards } => {
            let Some(p) = seat_of(player, seats) else {
                return;
            };
            if !ctx.core.player(p).alive {
                return;
            }
            let mut discarded = Vec::new();
            for c in cards {
                let id = c.raw();
                if id == 0 || discarded.contains(&id) {
                    continue;
                }
                if ctx.core.player(p).hand.contains(&id)
                    || ctx.core.player(p).equips.contains(&id)
                {
                    ctx.take_owned_card(p, id);
                    discarded.push(id);
                }
            }
            discard_some(ctx, p, discarded);
        }
        EffectRequest::StoreSelectedInSkillPile {
            player,
            cards,
            skill,
        } => {
            let Some(p) = seat_of(player, seats) else {
                return;
            };
            if !ctx.core.player(p).alive {
                return;
            }
            let pile_key =
                skill_label(ctx.catalog, skill.raw()).unwrap_or_else(|| executing.to_string());
            let mut chosen: Vec<CardId> = cards
                .into_iter()
                .map(|c| c.raw())
                .filter(|id| *id != 0)
                .collect();
            if chosen.is_empty() {
                if let Some(first) = ctx.core.player(p).hand.first().copied() {
                    chosen.push(first);
                }
            }
            let mut moved = Vec::new();
            for id in chosen {
                if moved.contains(&id) {
                    continue;
                }
                if ctx.core.player(p).hand.contains(&id)
                    || ctx.core.player(p).equips.contains(&id)
                {
                    ctx.take_owned_card(p, id);
                    push_pile(ctx, p, &pile_key, id);
                    moved.push(id);
                }
            }
        }
        EffectRequest::UseVirtualCardOn {
            player,
            kind,
            target,
        } => {
            let (Some(p), Some(t)) = (seat_of(player, seats), seat_of(target, seats)) else {
                return;
            };
            if !ctx.core.player(p).alive || !ctx.core.player(t).alive {
                return;
            }
            let Some(as_kind) = card_kind_from_tag(kind) else {
                return;
            };
            let nature = match as_kind {
                CardKind::FireSha => DamageNature::Fire,
                CardKind::LeiSha => DamageNature::Thunder,
                _ => DamageNature::Normal,
            };
            ctx.emit(EventData::CardUsed(crate::event::CardUseData {
                card: None,
                player: p,
                targets: vec![t],
                nature,
                use_as: Some(as_kind),
                cancelled: false,
            }));
        }
        EffectRequest::RecastCards { player, cards } => {
            let Some(p) = seat_of(player, seats) else {
                return;
            };
            if !ctx.core.player(p).alive {
                return;
            }
            let mut moved = Vec::new();
            for c in cards {
                let id = c.raw();
                if id == 0 || moved.contains(&id) {
                    continue;
                }
                if ctx.core.player(p).hand.contains(&id)
                    || ctx.core.player(p).equips.contains(&id)
                {
                    ctx.take_owned_card(p, id);
                    moved.push(id);
                }
            }
            let n = moved.len() as u32;
            discard_some(ctx, p, moved);
            if n > 0 {
                ctx.start_draw(p, n);
            }
        }
        EffectRequest::AddCardUsePolicySkill {
            player,
            card,
            policy_mask,
            skill_def,
        } => {
            let Some(p) = seat_of(player, seats) else {
                return;
            };
            if !ctx.core.player(p).alive {
                return;
            }
            if skill_def.is_some() {
                let raw = skill_def.raw();
                if let Some(def) = resolve_skill_def(ctx.catalog, raw) {
                    if !ctx.core.has_skill(p, def) {
                        ctx.add_skill_until_turn_end(p, def);
                    }
                } else {
                    ctx.core.vm_aux.flags.insert((p, format!("skill:{raw}")), 1);
                    ctx.core
                        .vm_aux
                        .flags
                        .insert((p, format!("skill_turn:{raw}")), 1);
                }
            }
            ctx.core
                .vm_aux
                .flags
                .insert((p, "card_policy".to_string()), u32::from(policy_mask));
            ctx.core.vm_aux.flags.insert(
                (p, format!("card_policy:{}", card.raw())),
                u32::from(policy_mask),
            );
            if (policy_mask & 0b11) != 0 {
                let cid = card.raw();
                for t in 0..seats {
                    ctx.core
                        .vm_aux
                        .flags
                        .insert((p, format!("sha_unrespondable:{cid}:{t}")), 1);
                }
            }
        }
        EffectRequest::DrawSelectedTargets { targets, count, .. } => {
            if count == 0 {
                return;
            }
            let living: Vec<PlayerId> = targets
                .into_iter()
                .filter_map(|t| seat_of(t, seats))
                .filter(|s| ctx.core.player(*s).alive)
                .collect();
            for s in living.into_iter().rev() {
                ctx.start_draw(s, count);
            }
        }
        EffectRequest::QueueExtraTurn {
            player,
            check,
            skill,
        } => {
            let Some(p) = seat_of(player, seats) else {
                return;
            };
            if !ctx.core.player(p).alive {
                return;
            }
            ctx.core
                .vm_aux
                .flags
                .insert((p, "extra_turn_queued".to_string()), 1);
            ctx.core
                .vm_aux
                .flags
                .insert((p, format!("extra_turn_check:{}", skill.raw())), check);
        }
        EffectRequest::DisableEquipSlots { player, slots } => {
            let Some(p) = seat_of(player, seats) else {
                return;
            };
            if !ctx.core.player(p).alive {
                return;
            }
            let mut mask = ctx
                .core
                .vm_aux
                .flags
                .get(&(p, "disabled_equip_slots".to_string()))
                .copied()
                .unwrap_or(0);
            let mut discarded = Vec::new();
            for s in slots {
                if !matches!(s, 1..=5) {
                    continue;
                }
                mask |= 1u32 << s;
                ctx.core
                    .vm_aux
                    .flags
                    .insert((p, format!("equip_slot_disabled:{s}")), 1);
                let want = match s {
                    1 => crate::catalog::EquipSlot::Weapon,
                    2 => crate::catalog::EquipSlot::Armor,
                    3 => crate::catalog::EquipSlot::HorsePlus,
                    4 => crate::catalog::EquipSlot::HorseMinus,
                    5 => crate::catalog::EquipSlot::Treasure,
                    _ => continue,
                };
                let matching: Vec<CardId> = ctx
                    .core
                    .player(p)
                    .equips
                    .iter()
                    .copied()
                    .filter(|c| ctx.catalog.card_slot_of(ctx.core, *c) == Some(want))
                    .collect();
                for c in matching {
                    if !discarded.contains(&c) {
                        ctx.take_owned_card(p, c);
                        discarded.push(c);
                    }
                }
            }
            ctx.core
                .vm_aux
                .flags
                .insert((p, "disabled_equip_slots".to_string()), mask);
            discard_some(ctx, p, discarded);
        }
        EffectRequest::EnableEquipSlot { player, slot, .. } => {
            let Some(p) = seat_of(player, seats) else {
                return;
            };
            if !matches!(slot, 1..=5) {
                return;
            }
            ctx.core
                .vm_aux
                .flags
                .remove(&(p, format!("equip_slot_disabled:{slot}")));
            let key = (p, "disabled_equip_slots".to_string());
            if let Some(m) = ctx.core.vm_aux.flags.get_mut(&key) {
                *m &= !(1u32 << slot);
                if *m == 0 {
                    ctx.core.vm_aux.flags.remove(&key);
                }
            }
        }
        EffectRequest::GainShadowCards { player, count, .. } => {
            let Some(p) = seat_of(player, seats) else {
                return;
            };
            if !ctx.core.player(p).alive || count == 0 {
                return;
            }
            *ctx.core
                .vm_aux
                .flags
                .entry((p, "shadow_cards_gained".to_string()))
                .or_insert(0) += count;
            ctx.start_draw(p, count);
        }
        EffectRequest::GainTargetHandCard { player, skill } => {
            let Some(p) = seat_of(player, seats) else {
                return;
            };
            if !ctx.core.player(p).alive {
                return;
            }
            let target_seat = ctx
                .core
                .vm_aux
                .selected_targets(p)
                .into_iter()
                .find_map(|raw| seat_of(rule_api::PlayerId::new(raw), seats))
                .or_else(|| {
                    ctx.core
                        .vm_aux
                        .flags
                        .get(&(p, format!("active_target:{executing}")))
                        .or_else(|| {
                            ctx.core
                                .vm_aux
                                .flags
                                .get(&(p, format!("active_target:{}", skill.raw())))
                        })
                        .or_else(|| ctx.core.vm_aux.flags.get(&(p, "active_target".to_string())))
                        .copied()
                        .and_then(|raw| seat_of(rule_api::PlayerId::new(raw), seats))
                })
                .or_else(|| {
                    ctx.core
                        .seat_order_from(p)
                        .into_iter()
                        .find(|&s| s != p && !ctx.core.player(s).hand.is_empty())
                });
            let Some(t) = target_seat else {
                return;
            };
            if t == p || !ctx.core.player(t).alive {
                return;
            }
            if let Some(card) = ctx.core.player(t).hand.first().copied() {
                ctx.give_hand_card(t, p, card);
            }
        }
        EffectRequest::AddBoundSkill {
            player,
            skill_def,
            bound_target,
        } => {
            let (Some(p), Some(_bt)) = (seat_of(player, seats), seat_of(bound_target, seats))
            else {
                return;
            };
            if !ctx.core.player(p).alive {
                return;
            }
            let raw = skill_def.raw();
            ctx.core
                .vm_aux
                .flags
                .insert((p, "bound_target".to_string()), bound_target.raw());
            ctx.core
                .vm_aux
                .flags
                .insert((p, format!("bound_target:{raw}")), bound_target.raw());
            if let Some(def) = resolve_skill_def(ctx.catalog, raw) {
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
            } else {
                ctx.core.vm_aux.flags.insert((p, format!("skill:{raw}")), 1);
            }
        }
        EffectRequest::GiveSelectedThenRecoverTarget {
            player,
            target,
            cards,
            amount,
            ready_to_recover,
            ..
        } => {
            let (Some(p), Some(t)) = (seat_of(player, seats), seat_of(target, seats)) else {
                return;
            };
            if !ctx.core.player(p).alive || !ctx.core.player(t).alive {
                return;
            }
            let mut valid = Vec::new();
            for c in cards {
                let id = c.raw();
                if id != 0
                    && !valid.contains(&id)
                    && ctx.core.player(p).hand.contains(&id)
                {
                    valid.push(id);
                }
            }
            if !valid.is_empty() && p != t {
                ctx.give_hand_cards(p, t, &valid);
            }
            if (ready_to_recover || !valid.is_empty()) && amount > 0 && ctx.core.player(t).alive {
                ctx.recover(t, amount, None);
            }
        }
        EffectRequest::DistributeShadowCards { player, count, .. } => {
            let Some(p) = seat_of(player, seats) else {
                return;
            };
            if !ctx.core.player(p).alive || count == 0 {
                return;
            }
            *ctx.core
                .vm_aux
                .flags
                .entry((p, "shadow_cards_distributed".to_string()))
                .or_insert(0) += count;
            ctx.start_draw(p, count);
        }
        EffectRequest::PlaceHandCardToExpansion { player, skill } => {
            let Some(p) = seat_of(player, seats) else {
                return;
            };
            if !ctx.core.player(p).alive {
                return;
            }
            let pile_key =
                skill_label(ctx.catalog, skill.raw()).unwrap_or_else(|| executing.to_string());
            if let Some(card) = ctx.core.player(p).hand.first().copied() {
                ctx.take_owned_card(p, card);
                push_pile(ctx, p, &pile_key, card);
            }
        }
        EffectRequest::RecordActiveTarget {
            player,
            target,
            skill,
        } => {
            let (Some(p), Some(_t)) = (seat_of(player, seats), seat_of(target, seats)) else {
                return;
            };
            ctx.core
                .vm_aux
                .flags
                .insert((p, "active_target".to_string()), target.raw());
            ctx.core
                .vm_aux
                .flags
                .insert((p, format!("active_target:{executing}")), target.raw());
            if skill.is_some() {
                ctx.core
                    .vm_aux
                    .flags
                    .insert((p, format!("active_target:{}", skill.raw())), target.raw());
            }
        }
        EffectRequest::ClearPhaseGiftTarget { player, skill } => {
            let Some(p) = seat_of(player, seats) else {
                return;
            };
            ctx.core
                .vm_aux
                .flags
                .remove(&(p, "phase_gift_target".to_string()));
            ctx.core
                .vm_aux
                .flags
                .remove(&(p, format!("phase_gift_target:{executing}")));
            if skill.is_some() {
                ctx.core
                    .vm_aux
                    .flags
                    .remove(&(p, format!("phase_gift_target:{}", skill.raw())));
            }
        }
        EffectRequest::DealDamageBanActiveIfDying {
            source,
            target,
            amount,
            skill,
        } => {
            let (Some(s), Some(t)) = (seat_of(source, seats), seat_of(target, seats)) else {
                return;
            };
            if !ctx.core.player(t).alive || amount == 0 {
                return;
            }
            if ctx.core.player(t).hp <= amount as i32 {
                ctx.core
                    .vm_aux
                    .once
                    .insert((s, executing.to_string(), 1));
                ctx.core
                    .vm_aux
                    .flags
                    .insert((s, "ban_active".to_string()), 1);
                ctx.core
                    .vm_aux
                    .flags
                    .insert((s, format!("skill_banned_turn:{executing}")), 1);
                if let Some(def) = resolve_skill_def(ctx.catalog, skill.raw()) {
                    let name = ctx.catalog.skill_name(def).to_string();
                    ctx.core.vm_aux.once.insert((s, name.clone(), 1));
                    ctx.core
                        .vm_aux
                        .flags
                        .insert((s, format!("skill_banned_turn:{name}")), 1);
                }
            }
            *ctx.core
                .vm_aux
                .flags
                .entry((s, "lifetime".to_string()))
                .or_insert(0) += amount;
            ctx.emit(EventData::Damage(crate::machine::damage_data_with(
                Some(s),
                t,
                amount as i32,
                DamageNature::Normal,
                None,
            )));
        }
        EffectRequest::DiscardSelectedDrawAllHandBonus { player, cards, .. } => {
            let Some(p) = seat_of(player, seats) else {
                return;
            };
            if !ctx.core.player(p).alive {
                return;
            }
            let hand_snapshot = ctx.core.player(p).hand.clone();
            let all_hand = !hand_snapshot.is_empty()
                && hand_snapshot
                    .iter()
                    .all(|h| cards.iter().any(|c| c.raw() == *h));
            let mut discarded = Vec::new();
            for c in cards {
                let id = c.raw();
                if id == 0 || discarded.contains(&id) {
                    continue;
                }
                if ctx.core.player(p).hand.contains(&id)
                    || ctx.core.player(p).equips.contains(&id)
                {
                    ctx.take_owned_card(p, id);
                    discarded.push(id);
                }
            }
            let draw_n = discarded.len() as u32 + u32::from(all_hand);
            discard_some(ctx, p, discarded);
            if draw_n > 0 {
                ctx.start_draw(p, draw_n);
            }
        }
        EffectRequest::DiscardSelectedRecordUniqueCategoryTarget {
            player,
            target,
            card,
            ..
        } => {
            let (Some(p), Some(_t)) = (seat_of(player, seats), seat_of(target, seats)) else {
                return;
            };
            if !ctx.core.player(p).alive {
                return;
            }
            ctx.core
                .vm_aux
                .flags
                .insert((p, "unique_cat_target".to_string()), target.raw());
            ctx.core.vm_aux.flags.insert(
                (p, format!("unique_cat_target:{executing}")),
                target.raw(),
            );
            let id = card.raw();
            if id != 0 {
                let cat_bit = if (id as usize) < ctx.catalog.cards.len() {
                    let k = ctx.catalog.card_kind(id);
                    if k.is_equip() {
                        4
                    } else if matches!(
                        k,
                        CardKind::Sha
                            | CardKind::Shan
                            | CardKind::Tao
                            | CardKind::Jiu
                            | CardKind::FireSha
                            | CardKind::LeiSha
                    ) {
                        1
                    } else {
                        2
                    }
                } else {
                    0
                };
                *ctx.core
                    .vm_aux
                    .flags
                    .entry((p, "unique_cat_mask".to_string()))
                    .or_insert(0) |= cat_bit;
                *ctx.core
                    .vm_aux
                    .flags
                    .entry((p, format!("unique_cat_mask:{executing}")))
                    .or_insert(0) |= cat_bit;
                if ctx.core.player(p).hand.contains(&id)
                    || ctx.core.player(p).equips.contains(&id)
                {
                    ctx.take_owned_card(p, id);
                    discard_some(ctx, p, vec![id]);
                }
            }
        }
        EffectRequest::DiscardTargetCardGainMount {
            player,
            target,
            scope,
        } => {
            let (Some(p), Some(t)) = (seat_of(player, seats), seat_of(target, seats)) else {
                return;
            };
            if !ctx.core.player(p).alive || !ctx.core.player(t).alive {
                return;
            }
            let picked = match scope {
                rule_api::TargetCardScope::Hand => ctx.core.player(t).hand.first().copied(),
                rule_api::TargetCardScope::Equip => ctx.core.player(t).equips.first().copied(),
                rule_api::TargetCardScope::Judge => ctx.core.player(t).judge.first().copied(),
                rule_api::TargetCardScope::EquipJudge => ctx
                    .core
                    .player(t)
                    .equips
                    .first()
                    .copied()
                    .or_else(|| ctx.core.player(t).judge.first().copied()),
                rule_api::TargetCardScope::HandEquip => ctx
                    .core
                    .player(t)
                    .hand
                    .first()
                    .copied()
                    .or_else(|| ctx.core.player(t).equips.first().copied()),
                rule_api::TargetCardScope::HandEquipJudge => ctx
                    .core
                    .player(t)
                    .hand
                    .first()
                    .copied()
                    .or_else(|| ctx.core.player(t).equips.first().copied())
                    .or_else(|| ctx.core.player(t).judge.first().copied()),
            };
            let Some(card) = picked else {
                return;
            };
            let is_mount = matches!(
                ctx.catalog.card_slot_of(ctx.core, card),
                Some(crate::catalog::EquipSlot::HorseMinus | crate::catalog::EquipSlot::HorsePlus)
            );
            if ctx.core.player(t).hand.contains(&card)
                || ctx.core.player(t).equips.contains(&card)
            {
                ctx.take_owned_card(t, card);
            } else if let Some(pos) = ctx.core.player(t).judge.iter().position(|c| *c == card) {
                ctx.core.player_mut(t).judge.remove(pos);
                ctx.core.declared_kinds.remove(&card);
            } else {
                return;
            }
            if is_mount {
                ctx.core.player_mut(p).hand.push(card);
                ctx.emit(EventData::CardsMoved {
                    cards: vec![card],
                    from: t,
                    to: p,
                });
            } else {
                discard_some(ctx, t, vec![card]);
            }
        }
        EffectRequest::DrawForLifetimeCardActionDivisors { player, divisors } => {
            let Some(p) = seat_of(player, seats) else {
                return;
            };
            if !ctx.core.player(p).alive {
                return;
            }
            let total = ctx
                .core
                .vm_aux
                .flags
                .get(&(p, "lifetime_card_actions".to_string()))
                .copied()
                .unwrap_or_else(|| {
                    ctx.core
                        .turn
                        .cards_used_play_phase
                        .max(ctx.core.turn.bte_used_play_phase)
                });
            if total == 0 {
                return;
            }
            let count = divisors
                .into_iter()
                .filter(|&d| d > 0 && total % d == 0)
                .count() as u32;
            if count > 0 {
                ctx.start_draw(p, count);
            }
        }
        EffectRequest::RecoverSelectedTargets {
            targets, amount, ..
        } => {
            if amount == 0 {
                return;
            }
            for t_pid in targets {
                if let Some(t) = seat_of(t_pid, seats) {
                    if ctx.core.player(t).alive {
                        ctx.recover(t, amount, None);
                    }
                }
            }
        }
        EffectRequest::RevealTopGainIfColor { player, color_mask } => {
            let Some(p) = seat_of(player, seats) else {
                return;
            };
            if !ctx.core.player(p).alive {
                return;
            }
            let Some(card) = pop_deck(ctx) else {
                return;
            };
            ctx.out.push(LogEvent::CardsRevealed {
                cards: vec![card],
                to: Vec::new(),
            });
            let suit_color = match ctx.catalog.cards[card as usize].suit {
                Suit::Heart | Suit::Diamond => 1u8,
                Suit::Spade | Suit::Club => 2u8,
            };
            if (color_mask & suit_color) != 0 {
                ctx.core.player_mut(p).hand.push(card);
                ctx.emit(EventData::CardsDrawn {
                    player: p,
                    cards: vec![card],
                });
                ctx.out.push(LogEvent::CardsDrawn {
                    player: p,
                    cards: vec![card],
                });
            } else {
                ctx.core.discard.push(card);
            }
        }
        EffectRequest::UseVirtualCardOnSelectedTargets {
            player,
            kind,
            targets,
        } => {
            let Some(p) = seat_of(player, seats) else {
                return;
            };
            if !ctx.core.player(p).alive {
                return;
            }
            let Some(as_kind) = card_kind_from_tag(kind) else {
                return;
            };
            let living_targets: Vec<PlayerId> = targets
                .into_iter()
                .filter_map(|t| seat_of(t, seats))
                .filter(|s| ctx.core.player(*s).alive)
                .collect();
            if living_targets.is_empty() {
                return;
            }
            let nature = match as_kind {
                CardKind::FireSha => DamageNature::Fire,
                CardKind::LeiSha => DamageNature::Thunder,
                _ => DamageNature::Normal,
            };
            if matches!(
                as_kind,
                CardKind::Duel | CardKind::Dismantle | CardKind::Snatch
            ) && living_targets.len() > 1
            {
                for target in living_targets.into_iter().rev() {
                    ctx.emit(EventData::CardUsed(crate::event::CardUseData {
                        card: None,
                        player: p,
                        targets: vec![target],
                        nature,
                        use_as: Some(as_kind),
                        cancelled: false,
                    }));
                }
            } else {
                ctx.emit(EventData::CardUsed(crate::event::CardUseData {
                    card: None,
                    player: p,
                    targets: living_targets,
                    nature,
                    use_as: Some(as_kind),
                    cancelled: false,
                }));
            }
        }
        _ => {}
    }
}
