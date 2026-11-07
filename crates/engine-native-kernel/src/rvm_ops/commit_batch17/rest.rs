//! PA12 批⑰后半部分（tags 239..=254 + ext_tag 256..=269）的权威状态落账。

use super::*;

pub(super) fn commit_batch17_rest(
    ctx: &mut Ctx<'_>,
    executing: &str,
    seats: u8,
    e: EffectRequest,
) {
    match e {
        EffectRequest::ResolveHoufeng {
            player,
            target,
            mode,
            ..
        } => {
            let Some(p) = seat_of(player, seats) else {
                return;
            };
            let t = if mode == 1 {
                p
            } else {
                seat_of(target, seats)
                    .or_else(|| first_other_alive(ctx.core, p))
                    .unwrap_or(p)
            };
            ctx.start_draw(t, 1);
            set_flag(ctx.core, p, "houfeng_mode", u32::from(mode) + 1);
        }
        EffectRequest::ResolveHuzun {
            player,
            action,
            targets,
            ..
        } => {
            let Some(p) = seat_of(player, seats) else {
                return;
            };
            if action == 0 {
                ctx.start_draw(p, 1);
                set_flag(ctx.core, p, "huzun_zhantao", 1);
            } else {
                if targets.is_empty() {
                    ctx.start_draw(p, 1);
                } else {
                    for tp in targets {
                        if let Some(q) = seat_of(tp, seats) {
                            ctx.start_draw(q, 1);
                        }
                    }
                }
                set_flag(ctx.core, p, "huzun_anjing", 1);
            }
        }
        EffectRequest::ResolveMalingli {
            player,
            helper_skill_def,
            action,
            target,
            ..
        } => {
            let Some(p) = seat_of(player, seats) else {
                return;
            };
            match action {
                0..=2 => {
                    if let Some(c) = pop_deck(ctx) {
                        let key = pile_name(ctx.catalog, executing, helper_skill_def.raw());
                        push_pile(ctx, p, &key, c);
                    }
                    set_flag(ctx.core, p, "malingli_prep", 1);
                }
                10..=14 => {
                    modify_current_damage(ctx, 1);
                    set_flag(ctx.core, p, "malingli_damage", 1);
                }
                20 => {
                    if let Some(t) = seat_of(target, seats).or_else(|| first_other_alive(ctx.core, p))
                    {
                        add_to_current_card_targets(ctx, 0, t);
                    }
                    set_flag(ctx.core, p, "malingli_rewrite", 1);
                }
                _ => {}
            }
        }
        EffectRequest::ResolveZhugemengxue {
            player,
            target,
            action,
            ..
        } => {
            let Some(p) = seat_of(player, seats) else {
                return;
            };
            match action {
                0..=4 => {
                    ctx.start_draw(p, 1);
                    set_flag(ctx.core, p, "zhugemengxue_jichun", u32::from(action) + 1);
                }
                10..=12 => {
                    let t = seat_of(target, seats).unwrap_or(p);
                    ctx.start_draw(t, 1);
                    set_flag(ctx.core, p, "zhugemengxue_hanying", u32::from(action));
                }
                _ => {}
            }
        }
        EffectRequest::ResolveTwyuyan {
            player,
            source,
            card,
            ..
        } => {
            let Some(p) = seat_of(player, seats) else {
                return;
            };
            if let Some(s) = seat_of(source, seats) {
                if let Some(&c) = ctx.core.player(s).hand.first() {
                    ctx.give_hand_card(s, p, c);
                    set_flag(ctx.core, p, "twyuyan_resolved", 1);
                    return;
                }
            }
            remove_from_current_card_targets(ctx, card.raw(), p);
            set_flag(ctx.core, p, "twyuyan_resolved", 1);
        }
        EffectRequest::ResolveTwzhibian { player, target, .. } => {
            let Some(p) = seat_of(player, seats) else {
                return;
            };
            let t = seat_of(target, seats)
                .or_else(|| first_other_alive(ctx.core, p))
                .unwrap_or(p);
            if let Some(&c) = ctx.core.player(t).hand.first() {
                ctx.discard_from_hand(t, c);
            }
            if is_wounded(ctx.core, p) {
                ctx.recover(p, 1, None);
            } else {
                ctx.start_draw(p, 1);
            }
            set_flag(ctx.core, p, "twzhibian_resolved", 1);
        }
        EffectRequest::ChooseAddSkillBoundCardTarget { player, card, .. } => {
            let Some(p) = seat_of(player, seats) else {
                return;
            };
            if let Some(t) = first_other_alive(ctx.core, p) {
                add_to_current_card_targets(ctx, card.raw(), t);
            }
            set_flag(ctx.core, p, "bound_card_target_added", 1);
        }
        EffectRequest::ChooseDiscardSkillPileCardAddMarkedSkill {
            player,
            target,
            helper_skill_def,
            selected,
            skill,
        } => {
            let Some(p) = seat_of(player, seats) else {
                return;
            };
            let key = pile_name(ctx.catalog, executing, skill.raw());
            let cid = selected
                .is_some()
                .then(|| selected.raw())
                .or_else(|| {
                    ctx.core
                        .vm_aux
                        .piles
                        .get(&(p, key.clone()))
                        .and_then(|v| v.first().copied())
                });
            if let Some(c) = cid {
                if detach(ctx.core, c) {
                    ctx.core.discard.push(c);
                }
            }
            let t = seat_of(target, seats).unwrap_or(p);
            grant_skill_raw(ctx, t, helper_skill_def.raw(), false);
            add_mark_raw(ctx.core, t, helper_skill_def.raw(), 1);
        }
        EffectRequest::ChooseDiscardTargetCardResolveCurrentSha {
            source,
            card,
            target,
        } => {
            let Some(s) = seat_of(source, seats) else {
                return;
            };
            let t = seat_of(target, seats)
                .or_else(|| first_other_alive(ctx.core, s))
                .unwrap_or(s);
            let _ = super::discard_one_scoped(ctx, t, rule_api::TargetCardScope::HandEquip);
            set_flag(ctx.core, s, &format!("sha_resolved:{}", card.raw()), 1);
        }
        EffectRequest::ChooseDiscardTargetCardResolveCurrentShaNonbasicBonus {
            source,
            card,
            target,
            bonus_skill_def,
        } => {
            let Some(s) = seat_of(source, seats) else {
                return;
            };
            let t = seat_of(target, seats)
                .or_else(|| first_other_alive(ctx.core, s))
                .unwrap_or(s);
            let _ = super::discard_one_scoped(ctx, t, rule_api::TargetCardScope::HandEquip);
            grant_skill_raw(ctx, s, bonus_skill_def.raw(), true);
            set_flag(ctx.core, s, &format!("sha_resolved:{}", card.raw()), 1);
        }
        EffectRequest::ChooseDiscardTargetCardsRecoverSameCategory {
            player,
            target,
            count,
            amount,
            ..
        } => {
            let Some(p) = seat_of(player, seats) else {
                return;
            };
            let t = seat_of(target, seats)
                .or_else(|| first_other_alive(ctx.core, p))
                .unwrap_or(p);
            let _ = super::discard_up_to_hand_equip(ctx, t, count);
            if is_wounded(ctx.core, p) {
                ctx.recover(p, amount, None);
            }
        }
        EffectRequest::ChooseGainCardFromEachOtherReplaceDraw {
            player,
            remaining_targets,
            ..
        } => {
            let Some(p) = seat_of(player, seats) else {
                return;
            };
            ctx.core.turn.skip_draw_phase = true;
            let targets: Vec<PlayerId> = if remaining_targets.is_empty() {
                seats_from(ctx.core, p)
                    .into_iter()
                    .filter(|&q| q != p && ctx.core.player(q).alive)
                    .collect()
            } else {
                remaining_targets
                    .into_iter()
                    .filter_map(|q| seat_of(q, seats))
                    .collect()
            };
            for q in targets {
                if let Some(&c) = ctx.core.player(q).hand.first() {
                    ctx.give_hand_card(q, p, c);
                } else if let Some(&c) = ctx.core.player(q).equips.first() {
                    ctx.take_owned_card(q, c);
                    ctx.core.player_mut(p).hand.push(c);
                }
            }
        }
        EffectRequest::ChooseGainCardFromEachOtherThenTurnOver {
            player,
            remaining_targets,
            ..
        }
        | EffectRequest::ChooseZonePriorityGainRandomCardFromEachOtherThenTurnOver {
            player,
            remaining_targets,
            ..
        } => {
            let Some(p) = seat_of(player, seats) else {
                return;
            };
            let targets: Vec<PlayerId> = if remaining_targets.is_empty() {
                seats_from(ctx.core, p)
                    .into_iter()
                    .filter(|&q| q != p && ctx.core.player(q).alive)
                    .collect()
            } else {
                remaining_targets
                    .into_iter()
                    .filter_map(|q| seat_of(q, seats))
                    .collect()
            };
            for q in targets {
                if let Some(&c) = ctx.core.player(q).hand.first() {
                    ctx.give_hand_card(q, p, c);
                } else if let Some(&c) = ctx.core.player(q).equips.first() {
                    ctx.take_owned_card(q, c);
                    ctx.core.player_mut(p).hand.push(c);
                }
            }
            ctx.emit(EventData::TurnOverBefore {
                player: p,
                cancelled: false,
            });
        }
        EffectRequest::ChooseGainOwnLostDiscardCardThisTurn { player, .. } => {
            let Some(p) = seat_of(player, seats) else {
                return;
            };
            if let Some(c) = ctx.core.discard.pop() {
                ctx.core.player_mut(p).hand.push(c);
            } else {
                ctx.start_draw(p, 1);
            }
        }
        EffectRequest::ChooseGainTargetCardAlternateZoneThenTargetDraw {
            player,
            target,
            selected,
            ..
        } => {
            let Some(p) = seat_of(player, seats) else {
                return;
            };
            let t = seat_of(target, seats)
                .or_else(|| first_other_alive(ctx.core, p))
                .unwrap_or(p);
            let cid = selected
                .is_some()
                .then(|| selected.raw())
                .or_else(|| ctx.core.player(t).hand.first().copied())
                .or_else(|| ctx.core.player(t).equips.first().copied());
            if let Some(c) = cid {
                if ctx.core.player(t).hand.contains(&c) {
                    ctx.give_hand_card(t, p, c);
                } else if ctx.core.player(t).equips.contains(&c) {
                    ctx.take_owned_card(t, c);
                    ctx.core.player_mut(p).hand.push(c);
                }
            }
            ctx.start_draw(t, 1);
        }
        EffectRequest::ChooseGainTopCardsBySuit {
            player,
            count,
            suit,
        } => {
            let Some(p) = seat_of(player, seats) else {
                return;
            };
            let want = match suit {
                rule_api::Suit::Spade => Some(Suit::Spade),
                rule_api::Suit::Heart => Some(Suit::Heart),
                rule_api::Suit::Club => Some(Suit::Club),
                rule_api::Suit::Diamond => Some(Suit::Diamond),
                rule_api::Suit::None => None,
            };
            let mut gained = 0;
            for _ in 0..count {
                let Some(c) = pop_deck(ctx) else {
                    break;
                };
                if want.is_none_or(|s| {
                    ctx.catalog
                        .cards
                        .get(c as usize)
                        .is_some_and(|cd| cd.suit == s)
                }) || gained == 0
                {
                    ctx.core.player_mut(p).hand.push(c);
                    gained += 1;
                } else {
                    ctx.core.discard.push(c);
                }
            }
        }
        EffectRequest::ChooseRevealHandCardBindCategoryTarget { player, target, .. } => {
            let Some(p) = seat_of(player, seats) else {
                return;
            };
            let t = seat_of(target, seats)
                .or_else(|| first_other_alive(ctx.core, p))
                .unwrap_or(p);
            set_flag(ctx.core, p, "bound_category_target", u32::from(t) + 1);
        }
        EffectRequest::ChooseRevealOtherHandCardRememberCategory { player, .. } => {
            let Some(p) = seat_of(player, seats) else {
                return;
            };
            set_flag(ctx.core, p, "remembered_category", 1);
        }
        EffectRequest::ChooseTargetHandCardToPile {
            player,
            skill,
            pile_def,
        } => {
            let Some(p) = seat_of(player, seats) else {
                return;
            };
            let t = first_other_alive(ctx.core, p).unwrap_or(p);
            if let Some(&c) = ctx.core.player(t).hand.first() {
                ctx.take_hand_card(t, c);
                let def = if pile_def != 0 { pile_def } else { skill.raw() };
                let key = pile_name(ctx.catalog, executing, def);
                push_pile(ctx, p, &key, c);
            }
        }
        EffectRequest::CityConsume { player, skill } => {
            let Some(p) = seat_of(player, seats) else {
                return;
            };
            let key = pile_name(ctx.catalog, executing, skill.raw());
            if let Some(pile) = ctx.core.vm_aux.piles.get_mut(&(p, key)) {
                if let Some(c) = pile.pop() {
                    ctx.core.discard.push(c);
                }
            }
            let t = first_other_alive(ctx.core, p).unwrap_or(p);
            ctx.start_draw(t, 1);
            set_flag(ctx.core, p, "city_consumed", 1);
        }
        EffectRequest::DiscardEquipmentPairWithArmor {
            player, targets, ..
        } => {
            let Some(p) = seat_of(player, seats) else {
                return;
            };
            let t = targets
                .first()
                .and_then(|&q| seat_of(q, seats))
                .or_else(|| first_other_alive(ctx.core, p))
                .unwrap_or(p);
            let _ = super::discard_one_scoped(ctx, p, rule_api::TargetCardScope::Equip);
            let _ = super::discard_one_scoped(ctx, t, rule_api::TargetCardScope::HandEquip);
            set_flag(ctx.core, p, "discard_equip_pair", 1);
        }
        EffectRequest::DiscardOneOwnXumou { player, .. } => {
            let Some(p) = seat_of(player, seats) else {
                return;
            };
            if let Some(&c) = ctx.core.player(p).judge.last() {
                ctx.take_owned_card(p, c);
                ctx.core.discard.push(c);
            }
            set_flag(ctx.core, p, "xumou_discarded", 1);
        }
        EffectRequest::DiscardRememberedHandCards { player, .. } => {
            let Some(p) = seat_of(player, seats) else {
                return;
            };
            if let Some(&c) = ctx.core.player(p).hand.first() {
                ctx.discard_from_hand(p, c);
            }
            set_flag(ctx.core, p, "remembered_hand_discarded", 1);
        }
        EffectRequest::DiscardTargetEquipmentDrawWeaponRange { player, target, .. } => {
            let Some(p) = seat_of(player, seats) else {
                return;
            };
            let t = seat_of(target, seats)
                .or_else(|| first_other_alive(ctx.core, p))
                .unwrap_or(p);
            let _ = super::discard_one_scoped(ctx, t, rule_api::TargetCardScope::Equip);
            ctx.start_draw(p, 1);
        }
        EffectRequest::ExtraResolveCurrentTrick { player, card, .. } => {
            let Some(p) = seat_of(player, seats) else {
                return;
            };
            set_flag(ctx.core, p, &format!("extra_resolve_trick:{}", card.raw()), 1);
            ctx.start_draw(p, 1);
        }
        EffectRequest::GainDrawCardsForLifetimeCardActionDivisors {
            player, divisors, ..
        } => {
            let Some(p) = seat_of(player, seats) else {
                return;
            };
            ctx.start_draw(p, 1);
            set_flag(ctx.core, p, "lifetime_divisor_draw", divisors[0]);
        }
        EffectRequest::GainHandCompareResultCard { player, cards, .. } => {
            let Some(p) = seat_of(player, seats) else {
                return;
            };
            let cid = cards
                .first()
                .map(|c| c.raw())
                .or_else(|| ctx.core.discard.last().copied());
            if let Some(c) = cid {
                if detach(ctx.core, c) {
                    ctx.core.player_mut(p).hand.push(c);
                }
            } else {
                ctx.start_draw(p, 1);
            }
            set_flag(ctx.core, p, "hand_compare_card_gained", 1);
        }
        EffectRequest::GainHighestRankShaFromEventCards { player, cards, .. } => {
            let Some(p) = seat_of(player, seats) else {
                return;
            };
            let best = cards
                .iter()
                .map(|c| c.raw())
                .filter(|&c| ctx.catalog.card_kind(c).is_sha_like())
                .max_by_key(|&c| {
                    ctx.catalog
                        .cards
                        .get(c as usize)
                        .map_or(0, |cd| cd.rank)
                })
                .or_else(|| cards.first().map(|c| c.raw()))
                .or_else(|| ctx.core.discard.last().copied());
            if let Some(c) = best {
                if detach(ctx.core, c) {
                    ctx.core.player_mut(p).hand.push(c);
                }
            }
            set_flag(ctx.core, p, "highest_sha_gained", 1);
        }
        EffectRequest::MoveEquipmentThenDraw { player, .. } => {
            let Some(p) = seat_of(player, seats) else {
                return;
            };
            ctx.start_draw(p, 1);
            set_flag(ctx.core, p, "move_equipment_then_draw", 1);
        }
        _ => {}
    }
}
