//! PA12 批⑩：区域整弃、武将复原、移除全部技能、连环群摸、先手额外出牌、
//! 牌堆装备武器、牌种移出与杀/伤害牌当次修正落账。

use super::commit::{detach, discard_some, push_pile};
use super::*;

pub(super) fn commit_combat_or_state_effect(
    ctx: &mut Ctx<'_>,
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
            let judge = std::mem::take(&mut ctx.core.player_mut(p).judge);
            for c in &judge {
                ctx.core.declared_kinds.remove(c);
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
            if !ctx.core.player(p).alive {
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
        _ => {}
    }
}
