//! PA12 批⑩：区域整弃、武将复原、移除全部技能、连环群摸、先手额外出牌、
//! 牌堆装备武器、牌种移出与杀/伤害牌当次修正落账。

use super::commit::{detach, discard_some, push_pile};
use super::*;

pub(super) fn commit_combat_or_state_effect(
    ctx: &mut Ctx<'_>,
    executing: &str,
    owner: PlayerId,
    seats: u8,
    e: EffectRequest,
) {
    match e {
        EffectRequest::DiscardAllHandAndEquip { player } => {
            let Some(p) = seat_of(player, seats) else {
                return;
            };
            if !ctx.core.player(p).alive {
                return;
            }
            let hand = ctx.core.player(p).hand.clone();
            let equips = ctx.core.player(p).equips.clone();
            for c in &hand {
                ctx.take_hand_card(p, *c);
            }
            for c in &equips {
                ctx.take_owned_card(p, *c);
            }
            let mut cards = hand;
            cards.extend(equips);
            discard_some(ctx, p, cards);
        }
        EffectRequest::DiscardAllHandEquipJudge { player } => {
            let Some(p) = seat_of(player, seats) else {
                return;
            };
            if !ctx.core.player(p).alive {
                return;
            }
            let hand = ctx.core.player(p).hand.clone();
            let equips = ctx.core.player(p).equips.clone();
            for c in &hand {
                ctx.take_hand_card(p, *c);
            }
            for c in &equips {
                ctx.take_owned_card(p, *c);
            }
            let judge = ctx.core.player(p).judge.clone();
            for c in &judge {
                ctx.core.declared_kinds.remove(c);
                ctx.take_owned_card(p, *c);
            }
            let mut cards = hand;
            cards.extend(equips);
            cards.extend(judge);
            discard_some(ctx, p, cards);
        }
        EffectRequest::RestoreCharacterState { player } => {
            let Some(p) = seat_of(player, seats) else {
                return;
            };
            if !ctx.core.player(p).alive {
                return;
            }
            if ctx.core.player(p).linked {
                ctx.emit(EventData::Linked {
                    player: p,
                    linked: false,
                });
            }
            if ctx.core.player(p).face_down {
                ctx.emit(EventData::TurnOverBefore {
                    player: p,
                    cancelled: false,
                });
            }
        }
        EffectRequest::RemoveAllSkills { player } => {
            let Some(p) = seat_of(player, seats) else {
                return;
            };
            let defs: Vec<u16> = ctx
                .core
                .skills
                .keys()
                .filter_map(|(o, d)| (*o == p).then_some(*d))
                .collect();
            for def in defs {
                ctx.remove_skill(p, def);
            }
            ctx.core.vm_aux.flags.retain(|(o, k), _| {
                *o != p || (!k.starts_with("skill:") && !k.starts_with("skill_turn:"))
            });
        }
        EffectRequest::DrawLinkedPlayers { count } => {
            let linked_seats: Vec<PlayerId> = ctx
                .core
                .seat_order_from(ctx.core.turn.current)
                .into_iter()
                .filter(|s| ctx.core.player(*s).alive && ctx.core.player(*s).linked)
                .collect();
            for p in linked_seats.into_iter().rev() {
                ctx.start_draw(p, count);
            }
        }
        EffectRequest::RecoverIfMinCurrentHp { player, amount } => {
            let Some(p) = seat_of(player, seats) else {
                return;
            };
            if ctx.core.player(p).alive
                && amount > 0
                && ctx
                    .core
                    .players
                    .iter()
                    .all(|o| !o.alive || ctx.core.player(p).hp <= o.hp)
            {
                ctx.recover(p, amount, None);
            }
        }
        EffectRequest::EquipWeaponFromDrawPile { player } => {
            let Some(p) = seat_of(player, seats) else {
                return;
            };
            if !ctx.core.player(p).alive
                || !has_enabled_equip_slot(ctx.core, p, crate::catalog::EquipSlot::Weapon)
            {
                return;
            }
            let has_weapon = ctx.core.player(p).equips.iter().any(|c| {
                ctx.catalog.card_slot_of(ctx.core, *c) == Some(crate::catalog::EquipSlot::Weapon)
            });
            if has_weapon {
                return;
            }
            if let Some(i) = ctx.core.deck.iter().rposition(|c| {
                ctx.catalog.card_slot_of(ctx.core, *c) == Some(crate::catalog::EquipSlot::Weapon)
            }) {
                let card = ctx.core.deck.remove(i);
                ctx.core.player_mut(p).equips.push(card);
                crate::kernel::grant_equip_skills(ctx, p, card);
                ctx.out.push(LogEvent::Equipped {
                    player: p,
                    card,
                    replaced: None,
                });
            }
        }
        EffectRequest::RemoveCardsByKind { kind } => {
            let matching: Vec<CardId> = ctx
                .core
                .deck
                .iter()
                .chain(ctx.core.discard.iter())
                .copied()
                .filter(|c| kind_tag(ctx.catalog.card_kind(*c)) == kind)
                .collect();
            for id in matching {
                if detach(ctx.core, id) {
                    push_pile(ctx, owner, "removed", id);
                }
            }
        }
        EffectRequest::QueueOpeningExtraPlayPhase { player, .. } => {
            let Some(p) = seat_of(player, seats) else {
                return;
            };
            if p == ctx.core.turn.current && ctx.core.player(p).alive {
                ctx.core.turn.extra_phases.push(2);
                ctx.core.turn.extra_phases.push(0);
            }
        }
        EffectRequest::IncreaseCurrentShaBaseDamage {
            player,
            card,
            amount,
            ..
        }
        | EffectRequest::IncreaseCurrentCardBaseDamage {
            player,
            card,
            amount,
            ..
        } => {
            let Some(p) = seat_of(player, seats) else {
                return;
            };
            if ctx.core.player(p).alive && amount > 0 {
                *ctx.core
                    .vm_aux
                    .flags
                    .entry((p, format!("card_damage_bonus:{}", card.raw())))
                    .or_insert(0) += amount;
            }
        }
        EffectRequest::IncreaseCurrentShaTargetDamage {
            source,
            card,
            target,
            amount,
        } => {
            let (Some(s), Some(t)) = (seat_of(source, seats), seat_of(target, seats)) else {
                return;
            };
            if ctx.core.player(s).alive && ctx.core.player(t).alive && amount > 0 {
                *ctx.core
                    .vm_aux
                    .flags
                    .entry((s, format!("sha_target_damage_bonus:{}:{t}", card.raw())))
                    .or_insert(0) += amount;
            }
        }
        EffectRequest::MakeCurrentShaTargetUnrespondable {
            source,
            card,
            target,
        } => {
            let (Some(s), Some(t)) = (seat_of(source, seats), seat_of(target, seats)) else {
                return;
            };
            if ctx.core.player(s).alive && ctx.core.player(t).alive {
                ctx.core
                    .vm_aux
                    .flags
                    .insert((s, format!("sha_unrespondable:{}:{t}", card.raw())), 1);
            }
        }
        EffectRequest::AdjustHandTo { player, count } => {
            let Some(p) = seat_of(player, seats) else {
                return;
            };
            if !ctx.core.player(p).alive {
                return;
            }
            let cur = ctx.core.player(p).hand.len();
            let want = count as usize;
            if cur < want {
                ctx.start_draw(p, (want - cur) as u32);
            } else if cur > want {
                let drop_n = cur - want;
                let cards: Vec<CardId> = ctx
                    .core
                    .player(p)
                    .hand
                    .iter()
                    .copied()
                    .take(drop_n)
                    .collect();
                for c in &cards {
                    ctx.take_hand_card(p, *c);
                }
                discard_some(ctx, p, cards);
            }
        }
        EffectRequest::DrawByPlayPhaseCardCategoryCount { player, .. } => {
            let Some(p) = seat_of(player, seats) else {
                return;
            };
            if !ctx.core.player(p).alive {
                return;
            }
            let mask = ctx
                .core
                .vm_aux
                .flags
                .get(&(p, "play_cat_mask".to_string()))
                .copied()
                .unwrap_or(0);
            let count = if mask != 0 {
                mask.count_ones()
            } else {
                ctx.core.turn.bte_used_play_phase.min(3)
            };
            if count > 0 {
                ctx.start_draw(p, count);
            }
        }
        EffectRequest::DrawEachBySeat {
            first,
            second,
            count,
            ..
        } => {
            let (Some(a), Some(b)) = (seat_of(first, seats), seat_of(second, seats)) else {
                return;
            };
            if count == 0 {
                return;
            }
            let order: Vec<PlayerId> = ctx
                .core
                .seat_order_from(ctx.core.turn.current)
                .into_iter()
                .filter(|s| (*s == a || *s == b) && ctx.core.player(*s).alive)
                .collect();
            for p in order.into_iter().rev() {
                ctx.start_draw(p, count);
            }
        }
        EffectRequest::SetDrawPhaseFromBottom { player } => {
            let Some(p) = seat_of(player, seats) else {
                return;
            };
            if ctx.core.player(p).alive {
                ctx.core
                    .vm_aux
                    .flags
                    .insert((p, "draw_phase_bottom".to_string()), 1);
            }
        }
        EffectRequest::DrawCardsFromBottom { player, count } => {
            let Some(p) = seat_of(player, seats) else {
                return;
            };
            if !ctx.core.player(p).alive || count == 0 {
                return;
            }
            let mut drawn = Vec::new();
            for _ in 0..count {
                if ctx.core.deck.is_empty() {
                    if ctx.core.discard.is_empty() {
                        break;
                    }
                    let mut cards = std::mem::take(&mut ctx.core.discard);
                    ctx.core.rng.shuffle(&mut cards);
                    ctx.core.deck = cards;
                    ctx.out.push(LogEvent::DeckReshuffled);
                }
                let card = ctx.core.deck.remove(0);
                ctx.core.player_mut(p).hand.push(card);
                drawn.push(card);
            }
            if !drawn.is_empty() {
                ctx.emit(EventData::CardsDrawn {
                    player: p,
                    cards: drawn.clone(),
                });
                ctx.out.push(LogEvent::CardsDrawn {
                    player: p,
                    cards: drawn,
                });
            }
        }
        EffectRequest::CancelIncomingCardUse {
            player,
            source,
            card,
            ..
        } => {
            let (Some(p), Some(s)) = (seat_of(player, seats), seat_of(source, seats)) else {
                return;
            };
            if !ctx.core.player(p).alive {
                return;
            }
            let cid = card.raw();
            let mut excluded = false;
            for frame in ctx.res.stack.iter_mut().rev() {
                if let EventData::CardUsed(u) = &mut frame.data {
                    if u.player == s && u.card == Some(cid) {
                        if let Some(pos) = u.targets.iter().position(|t| *t == p) {
                            u.targets.remove(pos);
                            excluded = true;
                        }
                        break;
                    }
                }
            }
            if excluded && cid != 0 {
                if let Some(pos) = ctx.core.table.iter().position(|x| *x == cid) {
                    ctx.core.table.remove(pos);
                    ctx.core.player_mut(p).hand.push(cid);
                    ctx.out.push(LogEvent::CardsDrawn {
                        player: p,
                        cards: vec![cid],
                    });
                } else if let Some(pos) = ctx.core.discard.iter().position(|x| *x == cid) {
                    ctx.core.discard.remove(pos);
                    ctx.core.player_mut(p).hand.push(cid);
                    ctx.out.push(LogEvent::CardsDrawn {
                        player: p,
                        cards: vec![cid],
                    });
                }
            }
        }
        EffectRequest::ExcludeSavedShaTarget { card, target } => {
            let Some(t) = seat_of(target, seats) else {
                return;
            };
            let cid = card.raw();
            let mut empty_after = false;
            for frame in ctx.res.stack.iter_mut().rev() {
                if let EventData::CardUsed(u) = &mut frame.data {
                    if u.card == Some(cid) {
                        u.targets.retain(|x| *x != t);
                        empty_after = u.targets.is_empty();
                        break;
                    }
                }
            }
            if empty_after && cid != 0 {
                if let Some(pos) = ctx.core.table.iter().position(|x| *x == cid) {
                    ctx.core.table.remove(pos);
                    ctx.core.discard.push(cid);
                }
            }
        }
        EffectRequest::TakeOneCard {
            from,
            to,
            equip_only,
        } => {
            let (Some(a), Some(b)) = (seat_of(from, seats), seat_of(to, seats)) else {
                return;
            };
            if a == b || !ctx.core.player(a).alive || !ctx.core.player(b).alive {
                return;
            }
            if !equip_only {
                if let Some(c) = ctx.core.player(a).hand.first().copied() {
                    ctx.give_hand_card(a, b, c);
                    return;
                }
            }
            if let Some(c) = ctx.core.player(a).equips.first().copied() {
                ctx.take_owned_card(a, c);
                ctx.core.player_mut(b).hand.push(c);
                ctx.emit(EventData::CardsMoved {
                    cards: vec![c],
                    from: a,
                    to: b,
                });
            }
        }
        EffectRequest::TakeOneHandCard { from, to } => {
            let (Some(a), Some(b)) = (seat_of(from, seats), seat_of(to, seats)) else {
                return;
            };
            if a == b || !ctx.core.player(a).alive || !ctx.core.player(b).alive {
                return;
            }
            if let Some(c) = ctx.core.player(a).hand.first().copied() {
                ctx.give_hand_card(a, b, c);
            }
        }
        EffectRequest::DisableSkillByDefUntilTurnEnd { player, skill_def } => {
            let Some(p) = seat_of(player, seats) else {
                return;
            };
            let raw = skill_def.raw();
            ctx.core
                .vm_aux
                .flags
                .insert((p, format!("skill_disabled_raw:{raw}")), 1);
            if let Some(def) = resolve_skill_def(ctx.catalog, raw) {
                if let Some(inst) = ctx.core.skills.get_mut(&(p, def)) {
                    if inst.enabled {
                        inst.enabled = false;
                        ctx.core
                            .vm_aux
                            .flags
                            .insert((p, format!("skill_disabled_turn:{def}")), 1);
                    }
                }
            }
        }
        EffectRequest::BanActiveUntilTurnEnd { player, skill } => {
            let Some(p) = seat_of(player, seats) else {
                return;
            };
            ctx.core
                .vm_aux
                .once
                .insert((p, executing.to_string(), 1));
            ctx.core
                .vm_aux
                .flags
                .insert((p, format!("skill_banned_turn:{executing}")), 1);
            if let Some(def) = resolve_skill_def(ctx.catalog, skill.raw()) {
                let name = ctx.catalog.skill_name(def).to_string();
                ctx.core.vm_aux.once.insert((p, name.clone(), 1));
                ctx.core
                    .vm_aux
                    .flags
                    .insert((p, format!("skill_banned_turn:{name}")), 1);
            }
        }
        EffectRequest::SetNonLockedSkillSuppression {
            player, enabled, ..
        } => {
            let Some(p) = seat_of(player, seats) else {
                return;
            };
            let key = (p, "non_locked_suppression_source".to_string());
            if enabled {
                if ctx.core.player(p).alive {
                    ctx.core.vm_aux.flags.insert(key, 1);
                }
            } else {
                ctx.core.vm_aux.flags.remove(&key);
            }
        }
        EffectRequest::SuppressNonLockedSkillsUntilTurnEnd { player } => {
            let Some(p) = seat_of(player, seats) else {
                return;
            };
            if ctx.core.player(p).alive {
                ctx.core
                    .vm_aux
                    .flags
                    .insert((p, "suppress_non_locked_turn".to_string()), 1);
            }
        }
        other => super::commit_ext::commit_ext_effect(ctx, executing, owner, seats, other),
    }
}
