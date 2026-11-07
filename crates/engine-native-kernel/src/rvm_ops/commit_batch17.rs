//! PA12 批⑰：60 种效果与具名复合结算器（tags 209..=238，其余委派 `rest`）的权威状态落账。

use super::commit::{detach, discard_some, pop_deck, push_pile};
use super::commit_batch15::{
    add_mark_raw, add_to_current_card_targets, deal_normal_damage, first_other_alive,
    grant_skill_raw, is_wounded, lose_hp_raw, modify_current_damage,
    remove_from_current_card_targets, remove_skill_raw, seats_from, set_flag,
};
use super::*;

mod rest;

fn discard_up_to_hand_equip(ctx: &mut Ctx<'_>, p: PlayerId, count: u32) -> usize {
    let mut picked = Vec::new();
    for &c in ctx
        .core
        .player(p)
        .hand
        .iter()
        .chain(ctx.core.player(p).equips.iter())
        .take(count as usize)
    {
        picked.push(c);
    }
    for &c in &picked {
        if ctx.core.player(p).hand.contains(&c) {
            ctx.take_hand_card(p, c);
        } else {
            ctx.take_owned_card(p, c);
        }
    }
    let n = picked.len();
    discard_some(ctx, p, picked);
    n
}

fn discard_one_scoped(ctx: &mut Ctx<'_>, t: PlayerId, scope: rule_api::TargetCardScope) -> bool {
    let candidate = match scope {
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
    let Some(c) = candidate else {
        return false;
    };
    if ctx.core.player(t).hand.contains(&c) {
        ctx.discard_from_hand(t, c);
    } else if ctx.core.player(t).equips.contains(&c) {
        ctx.take_owned_card(t, c);
        discard_some(ctx, t, vec![c]);
    } else if let Some(pos) = ctx.core.player(t).judge.iter().position(|&j| j == c) {
        ctx.core.player_mut(t).judge.remove(pos);
        discard_some(ctx, t, vec![c]);
    }
    true
}

pub(super) fn commit_batch17_effect(
    ctx: &mut Ctx<'_>,
    executing: &str,
    seats: u8,
    e: EffectRequest,
) {
    match e {
        EffectRequest::ChooseCardUserOrAddTarget { player, card, .. } => {
            let Some(p) = seat_of(player, seats) else {
                return;
            };
            if let Some(t) = first_other_alive(ctx.core, p) {
                add_to_current_card_targets(ctx, card.raw(), t);
            }
            ctx.start_draw(p, 1);
            set_flag(ctx.core, p, "choose_card_user_or_add_target", 1);
        }
        EffectRequest::ChooseCurrentCardUseEnhancements {
            player,
            card,
            helper_skill_def,
            ..
        } => {
            let Some(p) = seat_of(player, seats) else {
                return;
            };
            grant_skill_raw(ctx, p, helper_skill_def.raw(), true);
            set_flag(ctx.core, p, &format!("card_enhanced:{}", card.raw()), 1);
            set_flag(ctx.core, p, "current_card_enhanced", 1);
        }
        EffectRequest::ChooseDrawOrDiscardTargetCardRecordSha {
            player,
            card,
            target,
            ..
        } => {
            let Some(p) = seat_of(player, seats) else {
                return;
            };
            let t = seat_of(target, seats)
                .or_else(|| first_other_alive(ctx.core, p))
                .unwrap_or(p);
            if !discard_one_scoped(ctx, t, rule_api::TargetCardScope::HandEquip) {
                ctx.start_draw(p, 1);
            }
            set_flag(ctx.core, p, &format!("sha_recorded:{}", card.raw()), 1);
            set_flag(ctx.core, p, "record_sha_used", 1);
        }
        EffectRequest::ChooseOptionalDiscardTargetCard {
            player,
            target,
            scope,
        } => {
            let Some(p) = seat_of(player, seats) else {
                return;
            };
            let t = seat_of(target, seats)
                .or_else(|| first_other_alive(ctx.core, p))
                .unwrap_or(p);
            let _ = discard_one_scoped(ctx, t, scope);
            set_flag(ctx.core, p, "optional_discard_target_card", 1);
        }
        EffectRequest::ChooseUseVirtualCard { player, kind, .. } => {
            let Some(p) = seat_of(player, seats) else {
                return;
            };
            if !ctx.core.player(p).alive {
                return;
            }
            let as_kind = card_kind_from_tag(kind).unwrap_or(CardKind::Sha);
            let t = if matches!(as_kind, CardKind::Tao | CardKind::Jiu | CardKind::Wuzhong) {
                p
            } else {
                first_other_alive(ctx.core, p).unwrap_or(p)
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
        EffectRequest::UseVirtualShaFromCardColorLockSourceIfNoSha { player, .. } => {
            let Some(p) = seat_of(player, seats) else {
                return;
            };
            if let Some(&c) = ctx.core.player(p).hand.first() {
                ctx.discard_from_hand(p, c);
            }
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
            set_flag(ctx.core, p, "virtual_sha_color_lock", 1);
        }
        EffectRequest::ChooseDiscardHandEquipThenRecover {
            player,
            count,
            amount,
            ..
        } => {
            let Some(p) = seat_of(player, seats) else {
                return;
            };
            let _ = discard_up_to_hand_equip(ctx, p, count);
            if is_wounded(ctx.core, p) {
                ctx.recover(p, amount, None);
            }
        }
        EffectRequest::ChooseDiscardHandEquipThenRecoverTarget {
            chooser,
            target,
            count,
            amount,
            ..
        } => {
            let Some(p) = seat_of(chooser, seats) else {
                return;
            };
            let t = seat_of(target, seats).unwrap_or(p);
            let _ = discard_up_to_hand_equip(ctx, p, count);
            if is_wounded(ctx.core, t) {
                ctx.recover(t, amount, None);
            }
        }
        EffectRequest::ChooseDiscardWeaponOrLoseHp { player, .. } => {
            let Some(p) = seat_of(player, seats) else {
                return;
            };
            let weapon = ctx
                .core
                .player(p)
                .equips
                .iter()
                .copied()
                .find(|&c| {
                    ctx.catalog.card_slot_of(ctx.core, c)
                        == Some(crate::catalog::EquipSlot::Weapon)
                });
            if let Some(w) = weapon {
                ctx.take_owned_card(p, w);
                discard_some(ctx, p, vec![w]);
            } else {
                lose_hp_raw(ctx, p, 1);
            }
        }
        EffectRequest::ChooseMoveTargetEquipmentOrDraw { player, target, .. } => {
            let Some(p) = seat_of(player, seats) else {
                return;
            };
            let t = seat_of(target, seats)
                .or_else(|| first_other_alive(ctx.core, p))
                .unwrap_or(p);
            if let Some(&eq) = ctx.core.player(t).equips.first() {
                ctx.take_owned_card(t, eq);
                ctx.core.player_mut(p).hand.push(eq);
            } else {
                ctx.start_draw(p, 1);
            }
        }
        EffectRequest::ChooseOtherHandStoreSkillPile {
            player,
            target,
            skill,
        } => {
            let Some(p) = seat_of(player, seats) else {
                return;
            };
            let t = seat_of(target, seats)
                .or_else(|| first_other_alive(ctx.core, p))
                .unwrap_or(p);
            if let Some(&c) = ctx.core.player(t).hand.first() {
                ctx.take_hand_card(t, c);
                let key = pile_name(ctx.catalog, executing, skill.raw());
                push_pile(ctx, p, &key, c);
            }
        }
        EffectRequest::ChooseOtherYinghun {
            player,
            target,
            count,
            branch,
            ..
        } => {
            let Some(p) = seat_of(player, seats) else {
                return;
            };
            let t = seat_of(target, seats)
                .or_else(|| first_other_alive(ctx.core, p))
                .unwrap_or(p);
            let lost = ctx
                .core
                .player(p)
                .max_hp
                .saturating_sub(ctx.core.player(p).hp.max(0) as u32);
            let x = count.max(lost).max(1);
            if branch == 1 {
                ctx.start_draw(t, 1);
                let _ = discard_up_to_hand_equip(ctx, t, x);
            } else {
                ctx.start_draw(t, x);
                let _ = discard_up_to_hand_equip(ctx, t, 1);
            }
            set_flag(ctx.core, p, "yinghun_resolved", 1);
        }
        EffectRequest::ChooseTargetDiscardExcessThenSelfDiscardOrTargetDraw { player, .. } => {
            let Some(p) = seat_of(player, seats) else {
                return;
            };
            let t = first_other_alive(ctx.core, p).unwrap_or(p);
            if ctx.core.player(t).hand.len() > ctx.core.player(p).hand.len() {
                let diff = (ctx.core.player(t).hand.len() - ctx.core.player(p).hand.len()) as u32;
                let _ = discard_up_to_hand_equip(ctx, t, diff);
            }
            if !discard_one_scoped(ctx, p, rule_api::TargetCardScope::HandEquip) {
                ctx.start_draw(t, 1);
            }
            set_flag(ctx.core, p, "discard_excess_resolved", 1);
        }
        EffectRequest::ResolveDiliYuheng {
            player,
            action,
            card,
            event_cards,
            ..
        } => {
            let Some(p) = seat_of(player, seats) else {
                return;
            };
            if action == 0 {
                add_mark_raw(ctx.core, p, 1, 1);
                set_flag(ctx.core, p, "dili_triggered", 1);
            } else {
                let cid = card
                    .is_some()
                    .then(|| card.raw())
                    .or_else(|| event_cards.first().map(|c| c.raw()));
                if let Some(c) = cid {
                    if detach(ctx.core, c) {
                        ctx.core.player_mut(p).hand.push(c);
                        set_flag(ctx.core, p, "yuheng_resolved", u32::from(action));
                        return;
                    }
                }
                ctx.start_draw(p, 1);
                set_flag(ctx.core, p, "yuheng_resolved", u32::from(action));
            }
        }
        EffectRequest::ChooseOtherTargetsBySkippedPhaseCountDamage { player, amount, .. } => {
            let Some(p) = seat_of(player, seats) else {
                return;
            };
            if let Some(t) = first_other_alive(ctx.core, p) {
                deal_normal_damage(ctx, Some(p), t, amount as i32);
            }
        }
        EffectRequest::GiveAllEquipmentChooseRecoverOrDamage { player, target, .. } => {
            let Some(p) = seat_of(player, seats) else {
                return;
            };
            let t = seat_of(target, seats)
                .or_else(|| first_other_alive(ctx.core, p))
                .unwrap_or(p);
            let eqs: Vec<u32> = ctx.core.player(p).equips.clone();
            for c in eqs {
                ctx.take_owned_card(p, c);
                ctx.core.player_mut(t).hand.push(c);
            }
            if is_wounded(ctx.core, p) {
                ctx.recover(p, 1, None);
            } else if t != p {
                deal_normal_damage(ctx, Some(p), t, 1);
            }
        }
        EffectRequest::LoseHpDoubleCurrentDamageThenReplace {
            player,
            skill,
            replacement_skill_def,
            ..
        } => {
            let Some(p) = seat_of(player, seats) else {
                return;
            };
            lose_hp_raw(ctx, p, 1);
            modify_current_damage(ctx, 1);
            remove_skill_raw(ctx, p, skill.raw());
            grant_skill_raw(ctx, p, replacement_skill_def.raw(), false);
        }
        EffectRequest::DrawThenOptionallyDistributeHandCards {
            player,
            draw_count,
            give_cap,
            cards,
            targets,
            ..
        } => {
            let Some(p) = seat_of(player, seats) else {
                return;
            };
            ctx.start_draw(p, draw_count);
            if let Some(t) = targets.first().and_then(|&q| seat_of(q, seats)) {
                for c in cards.into_iter().take(give_cap as usize) {
                    let raw = c.raw();
                    if ctx.core.player(p).hand.contains(&raw) {
                        ctx.give_hand_card(p, t, raw);
                    }
                }
            }
        }
        EffectRequest::ReplaceDrawWithDistinctSuitSelection {
            player,
            count,
            cards,
            ..
        } => {
            let Some(p) = seat_of(player, seats) else {
                return;
            };
            ctx.core.turn.skip_draw_phase = true;
            if !cards.is_empty() {
                for c in cards {
                    let raw = c.raw();
                    if detach(ctx.core, raw) {
                        ctx.core.player_mut(p).hand.push(raw);
                    }
                }
            } else {
                ctx.start_draw(p, u32::from(count.max(1)));
            }
        }
        EffectRequest::StartDebateAllOthers { player, .. } => {
            let Some(p) = seat_of(player, seats) else {
                return;
            };
            set_flag(ctx.core, p, "debate_resolved", 1);
            ctx.start_draw(p, 1);
        }
        EffectRequest::StartDebateTargets { player, .. } => {
            let Some(p) = seat_of(player, seats) else {
                return;
            };
            set_flag(ctx.core, p, "debate_targets_resolved", 1);
            ctx.start_draw(p, 1);
        }
        EffectRequest::ResolveBailingyun {
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
                    set_flag(ctx.core, p, "bailingyun_linghui", u32::from(action) + 1);
                }
                10 => {
                    if let Some(t) = seat_of(target, seats).or_else(|| first_other_alive(ctx.core, p))
                    {
                        deal_normal_damage(ctx, Some(p), t, 1);
                    }
                    set_flag(ctx.core, p, "bailingyun_xiace", 10);
                }
                11 => {
                    if is_wounded(ctx.core, p) {
                        ctx.recover(p, 1, None);
                    } else {
                        ctx.start_draw(p, 1);
                    }
                    set_flag(ctx.core, p, "bailingyun_xiace", 11);
                }
                _ => {}
            }
        }
        EffectRequest::ResolveJieliangGain { player, cards, .. } => {
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
            set_flag(ctx.core, p, "jieliang_gained", 1);
        }
        EffectRequest::ResolveJsrgchaozheng { player, .. } => {
            let Some(p) = seat_of(player, seats) else {
                return;
            };
            if is_wounded(ctx.core, p) {
                ctx.recover(p, 1, None);
            } else {
                ctx.start_draw(p, 1);
            }
            set_flag(ctx.core, p, "jsrgchaozheng_resolved", 1);
        }
        EffectRequest::ResolveStarcuxiaWeiming {
            player,
            source,
            variant,
        } => {
            let Some(p) = seat_of(player, seats) else {
                return;
            };
            if let Some(s) = seat_of(source, seats) {
                if let Some(&c) = ctx.core.player(s).hand.first() {
                    ctx.discard_from_hand(s, c);
                }
            }
            ctx.start_draw(p, 1);
            set_flag(ctx.core, p, "starcuxia_weiming", variant + 1);
        }
        EffectRequest::ResolveReanxu {
            player,
            first,
            second,
            card,
            ..
        } => {
            let Some(p) = seat_of(player, seats) else {
                return;
            };
            let f = seat_of(first, seats)
                .or_else(|| first_other_alive(ctx.core, p))
                .unwrap_or(p);
            let s = seat_of(second, seats).unwrap_or(p);
            let cid = card
                .is_some()
                .then(|| card.raw())
                .or_else(|| ctx.core.player(s).hand.first().copied())
                .or_else(|| ctx.core.player(s).equips.first().copied());
            if let Some(c) = cid {
                if ctx.core.player(s).hand.contains(&c) {
                    ctx.give_hand_card(s, f, c);
                } else if ctx.core.player(s).equips.contains(&c) {
                    ctx.take_owned_card(s, c);
                    ctx.core.player_mut(f).hand.push(c);
                }
            }
            ctx.start_draw(p, 1);
            set_flag(ctx.core, p, "reanxu_resolved", 1);
        }
        EffectRequest::ResolveRebingyi { player, .. } => {
            let Some(p) = seat_of(player, seats) else {
                return;
            };
            let n = (ctx.core.player(p).hand.len() as u32).clamp(1, 3);
            ctx.start_draw(p, n);
            set_flag(ctx.core, p, "rebingyi_resolved", 1);
        }
        EffectRequest::ResolveRejizhi { player, card, .. } => {
            let Some(p) = seat_of(player, seats) else {
                return;
            };
            ctx.start_draw(p, 1);
            if card.is_some() {
                set_flag(ctx.core, p, "rejizhi_card", card.raw());
            }
            set_flag(ctx.core, p, "rejizhi_resolved", 1);
        }
        EffectRequest::ForceOthersUseShaOnNearestOrLoseHp {
            player, remaining, ..
        } => {
            let Some(p) = seat_of(player, seats) else {
                return;
            };
            let list: Vec<PlayerId> = if remaining.is_empty() {
                seats_from(ctx.core, p)
                    .into_iter()
                    .filter(|&q| q != p && ctx.core.player(q).alive)
                    .collect()
            } else {
                remaining
                    .into_iter()
                    .filter_map(|q| seat_of(q, seats))
                    .filter(|&q| ctx.core.player(q).alive)
                    .collect()
            };
            for q in list {
                let sha = ctx
                    .core
                    .player(q)
                    .hand
                    .iter()
                    .copied()
                    .find(|&c| ctx.catalog.card_kind(c).is_sha_like());
                if let Some(c) = sha {
                    ctx.discard_from_hand(q, c);
                } else {
                    lose_hp_raw(ctx, q, 1);
                }
            }
        }
        EffectRequest::TargetDiscardThenDuelIfNonShaAndHpGteOwner { player, target, .. } => {
            let Some(p) = seat_of(player, seats) else {
                return;
            };
            let t = seat_of(target, seats)
                .or_else(|| first_other_alive(ctx.core, p))
                .unwrap_or(p);
            if let Some(&c) = ctx.core.player(t).hand.first() {
                let is_sha = ctx.catalog.card_kind(c).is_sha_like();
                ctx.discard_from_hand(t, c);
                if !is_sha && ctx.core.player(t).hp >= ctx.core.player(p).hp && t != p {
                    ctx.emit(EventData::CardUsed(crate::event::CardUseData {
                        card: None,
                        player: p,
                        targets: vec![t],
                        nature: DamageNature::Normal,
                        use_as: Some(CardKind::Duel),
                        cancelled: false,
                    }));
                }
            }
        }
        other => rest::commit_batch17_rest(ctx, executing, seats, other),
    }
}
