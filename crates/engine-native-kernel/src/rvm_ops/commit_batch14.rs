//! PA12 批⑭：25 种高频交互式原语（tags 129..=153）无 `vm_commit` 钩子时的确定性回退落账。

use super::commit::discard_some;
use super::*;

pub(super) fn commit_batch14_effect(
    ctx: &mut Ctx<'_>,
    executing: &str,
    owner: PlayerId,
    seats: u8,
    e: EffectRequest,
) {
    let _ = owner;
    match e {
        EffectRequest::ChooseOtherDrawOrRecover {
            player,
            draw_count,
            recover_count,
            times,
            ..
        } => {
            let Some(p) = seat_of(player, seats) else {
                return;
            };
            let Some(t) = first_other_alive(ctx.core, p) else {
                return;
            };
            for _ in 0..times.min(8) {
                if recover_count > 0 && is_wounded(ctx.core, t) {
                    ctx.recover(t, recover_count, None);
                } else if draw_count > 0 {
                    ctx.start_draw(t, draw_count);
                }
            }
        }
        EffectRequest::ChooseAddOrRemoveCurrentCardTarget { player, card, .. } => {
            let Some(p) = seat_of(player, seats) else {
                return;
            };
            let cid = card.raw();
            let cand = seats_from(ctx.core, p)
                .into_iter()
                .find(|&q| q != p && ctx.core.player(q).alive);
            for frame in ctx.res.stack.iter_mut().rev() {
                if let EventData::CardUsed(u) = &mut frame.data {
                    if cid == 0 || u.card == Some(cid) {
                        if let Some(q) = cand.filter(|q| !u.targets.contains(q)) {
                            u.targets.push(q);
                        } else if u.targets.len() > 1 {
                            u.targets.pop();
                        }
                        break;
                    }
                }
            }
        }
        EffectRequest::ChooseCompareSelectedHandCard {
            player,
            target,
            card,
            win_skill_def,
            lose_skill_def,
            ..
        } => {
            let Some(p) = seat_of(player, seats) else {
                return;
            };
            let t = seat_of(target, seats).or_else(|| first_other_alive(ctx.core, p));
            let Some(t) = t else {
                return;
            };
            run_fallback_pindian(ctx, executing, p, t, card, win_skill_def, lose_skill_def);
        }
        EffectRequest::ChooseTargetsDamage {
            player,
            min_count,
            max_count,
            scope,
            amount,
            ..
        } => {
            let Some(p) = seat_of(player, seats) else {
                return;
            };
            let want = usize::from(min_count.max(1).min(max_count));
            let targets: Vec<PlayerId> = scope_candidates(ctx.core, p, scope)
                .into_iter()
                .take(want)
                .collect();
            for t in targets.into_iter().rev() {
                ctx.emit(EventData::Damage(crate::machine::damage_data_with(
                    Some(p),
                    t,
                    amount as i32,
                    DamageNature::Normal,
                    None,
                )));
            }
        }
        EffectRequest::ChooseAddCurrentShaTarget { player, card, .. } => {
            let Some(p) = seat_of(player, seats) else {
                return;
            };
            let cid = card.raw();
            let all_alive: Vec<PlayerId> = seats_from(ctx.core, p)
                .into_iter()
                .filter(|&q| q != p && ctx.core.player(q).alive)
                .collect();
            for frame in ctx.res.stack.iter_mut().rev() {
                if let EventData::CardUsed(u) = &mut frame.data {
                    if cid == 0 || u.card == Some(cid) {
                        if let Some(q) = all_alive.into_iter().find(|q| !u.targets.contains(q)) {
                            u.targets.push(q);
                        }
                        break;
                    }
                }
            }
        }
        EffectRequest::DrawThenGiveOne {
            player,
            target,
            count,
            ..
        } => {
            let (Some(p), Some(t)) = (seat_of(player, seats), seat_of(target, seats)) else {
                return;
            };
            if !ctx.core.player(p).alive {
                return;
            }
            if p != t && ctx.core.player(t).alive {
                if let Some(c) = ctx.core.player(p).hand.first().copied() {
                    ctx.give_hand_card(p, t, c);
                }
            }
            if count > 0 {
                ctx.start_draw(p, count);
            }
        }
        EffectRequest::ChooseHandCompare {
            player,
            win_skill_def,
            lose_skill_def,
            ..
        } => {
            let Some(p) = seat_of(player, seats) else {
                return;
            };
            let Some(t) = seats_from(ctx.core, p)
                .into_iter()
                .find(|&q| q != p && ctx.core.player(q).alive && !ctx.core.player(q).hand.is_empty())
            else {
                return;
            };
            run_fallback_pindian(
                ctx,
                executing,
                p,
                t,
                rule_api::CardId::NONE,
                win_skill_def,
                lose_skill_def,
            );
        }
        EffectRequest::ChooseOptionalTargetsDraw {
            player,
            count,
            min_count,
            max_count,
            scope,
            ..
        } => {
            let Some(p) = seat_of(player, seats) else {
                return;
            };
            let want = usize::from(min_count.max(1).min(max_count));
            let targets: Vec<PlayerId> = scope_candidates(ctx.core, p, scope)
                .into_iter()
                .take(want)
                .collect();
            for t in targets.into_iter().rev() {
                ctx.start_draw(t, count);
            }
        }
        EffectRequest::ChooseTargetsDraw {
            player,
            count,
            target_count,
            scope,
            ..
        } => {
            let Some(p) = seat_of(player, seats) else {
                return;
            };
            let want = usize::from(target_count);
            let targets: Vec<PlayerId> = scope_candidates(ctx.core, p, scope)
                .into_iter()
                .take(want)
                .collect();
            for t in targets.into_iter().rev() {
                ctx.start_draw(t, count);
            }
        }
        EffectRequest::GuanxingTop { player, count, .. } => {
            let Some(p) = seat_of(player, seats) else {
                return;
            };
            if !ctx.core.player(p).alive {
                return;
            }
            let n = (count as usize).min(5);
            let mut peeked = Vec::new();
            for _ in 0..n {
                if let Some(c) = super::commit::pop_deck(ctx) {
                    peeked.push(c);
                }
            }
            if !peeked.is_empty() {
                ctx.core.deck.extend(peeked.into_iter().rev());
            }
        }
        EffectRequest::ChooseLinkOther { player, .. } => {
            let Some(p) = seat_of(player, seats) else {
                return;
            };
            let target = seats_from(ctx.core, p)
                .into_iter()
                .find(|&q| q != p && ctx.core.player(q).alive && !ctx.core.player(q).linked)
                .or_else(|| first_other_alive(ctx.core, p));
            if let Some(t) = target {
                let next = !ctx.core.player(t).linked;
                ctx.core.player_mut(t).linked = next;
                ctx.emit(EventData::Linked {
                    player: t,
                    linked: next,
                });
            }
        }
        EffectRequest::ChooseOtherDrawOwnerLostHpTurnOver { player, .. } => {
            let Some(p) = seat_of(player, seats) else {
                return;
            };
            let Some(t) = first_other_alive(ctx.core, p) else {
                return;
            };
            let lost = (ctx.core.player(p).max_hp as i32 - ctx.core.player(p).hp).max(0) as u32;
            ctx.emit(EventData::TurnOverBefore {
                player: p,
                cancelled: false,
            });
            if lost > 0 {
                ctx.start_draw(t, lost);
            }
        }
        EffectRequest::ChooseOneFromSkillPileDiscard {
            player, pile_def, ..
        } => {
            let Some(p) = seat_of(player, seats) else {
                return;
            };
            let pname = pile_name(ctx.catalog, executing, pile_def);
            let card = ctx
                .core
                .vm_aux
                .piles
                .get_mut(&(p, pname))
                .and_then(|v| (!v.is_empty()).then(|| v.remove(0)));
            if let Some(c) = card {
                discard_some(ctx, p, vec![c]);
            }
        }
        EffectRequest::ChooseAddCurrentShaTargetWithinDistance {
            player,
            card,
            max_distance,
            ..
        } => {
            let Some(p) = seat_of(player, seats) else {
                return;
            };
            let cid = card.raw();
            let cands: Vec<PlayerId> = seats_from(ctx.core, p)
                .into_iter()
                .filter(|&q| {
                    q != p
                        && ctx.core.player(q).alive
                        && ctx.core.computed_distance(ctx.catalog, p, q) <= max_distance
                })
                .collect();
            for frame in ctx.res.stack.iter_mut().rev() {
                if let EventData::CardUsed(u) = &mut frame.data {
                    if cid == 0 || u.card == Some(cid) {
                        if let Some(q) = cands.into_iter().find(|q| !u.targets.contains(q)) {
                            u.targets.push(q);
                        }
                        break;
                    }
                }
            }
        }
        EffectRequest::ChooseRecoverTarget {
            chooser,
            target,
            amount,
        } => {
            let (Some(_c), Some(t)) = (seat_of(chooser, seats), seat_of(target, seats)) else {
                return;
            };
            if ctx.core.player(t).alive && is_wounded(ctx.core, t) {
                ctx.recover(t, amount, None);
            }
        }
        EffectRequest::ChooseTargetDrawToMaxHpCap { player, cap, .. } => {
            let Some(p) = seat_of(player, seats) else {
                return;
            };
            for q in seats_from(ctx.core, p) {
                if !ctx.core.player(q).alive {
                    continue;
                }
                let goal = ctx.core.player(q).max_hp.min(cap) as usize;
                let cur = ctx.core.player(q).hand.len();
                if cur < goal {
                    ctx.start_draw(q, (goal - cur) as u32);
                    break;
                }
            }
        }
        EffectRequest::ChooseDiscardHandRange {
            player, min, max, ..
        } => {
            let Some(p) = seat_of(player, seats) else {
                return;
            };
            let want = min.max(1).min(max) as usize;
            let take: Vec<CardId> = ctx
                .core
                .player(p)
                .hand
                .iter()
                .copied()
                .take(want)
                .collect();
            for &c in &take {
                ctx.take_owned_card(p, c);
            }
            discard_some(ctx, p, take);
        }
        EffectRequest::ChooseDiscardHandEquipThenRemoveSelfSkill {
            player,
            count,
            skill,
        } => {
            let Some(p) = seat_of(player, seats) else {
                return;
            };
            let want = count as usize;
            let mut take: Vec<CardId> = ctx
                .core
                .player(p)
                .hand
                .iter()
                .copied()
                .chain(ctx.core.player(p).equips.iter().copied())
                .take(want)
                .collect();
            take.truncate(want);
            for &c in &take {
                ctx.take_owned_card(p, c);
            }
            discard_some(ctx, p, take);
            if let Some(def) = resolve_skill_def(ctx.catalog, skill.raw())
                .or_else(|| ctx.catalog.skill_by_name(executing))
            {
                ctx.remove_skill(p, def);
            }
        }
        EffectRequest::ChooseHandCardsToSkillPile {
            player, min_count, max_count, ..
        } => {
            let Some(p) = seat_of(player, seats) else {
                return;
            };
            let want = min_count.max(1).min(max_count) as usize;
            let take: Vec<CardId> = ctx
                .core
                .player(p)
                .hand
                .iter()
                .copied()
                .take(want)
                .collect();
            if take.is_empty() {
                return;
            }
            for &c in &take {
                ctx.take_owned_card(p, c);
            }
            ctx.core
                .vm_aux
                .piles
                .entry((p, executing.to_string()))
                .or_default()
                .extend(take);
        }
        EffectRequest::ChooseDiscardEquipmentPreventCurrentDamage { player, .. } => {
            let Some(p) = seat_of(player, seats) else {
                return;
            };
            let Some(c) = ctx.core.player(p).equips.first().copied() else {
                return;
            };
            ctx.take_owned_card(p, c);
            discard_some(ctx, p, vec![c]);
            for frame in ctx.res.stack.iter_mut().rev() {
                if let EventData::Damage(d) = &mut frame.data {
                    d.prevented = true;
                    d.amount = 0;
                    break;
                }
            }
        }
        EffectRequest::ChooseGainDiscardCardByKind { player, kind, .. } => {
            let Some(p) = seat_of(player, seats) else {
                return;
            };
            let pos = ctx.core.discard.iter().position(|&c| {
                let k = ctx.catalog.cards[c as usize].kind;
                match kind {
                    100 => k.is_basic_kind(),
                    101 => k.is_basic_trick_or_equip() && !k.is_basic_kind() && !k.is_equip(),
                    102 => k.is_equip(),
                    tag => kind_tag(k) == tag || card_kind_from_tag(tag) == Some(k),
                }
            });
            if let Some(idx) = pos {
                let c = ctx.core.discard.remove(idx);
                ctx.core.player_mut(p).hand.push(c);
                ctx.out.push(LogEvent::CardsDrawn {
                    player: p,
                    cards: vec![c],
                });
            }
        }
        EffectRequest::ChooseAddMissingSkill {
            player, skill_defs, ..
        } => {
            let Some(p) = seat_of(player, seats) else {
                return;
            };
            for def_id in skill_defs {
                if !def_id.is_some() {
                    continue;
                }
                if let Some(def) = resolve_skill_def(ctx.catalog, def_id.raw()) {
                    if !ctx.core.has_skill(p, def) {
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
                        break;
                    }
                }
            }
        }
        EffectRequest::ChooseMoveFieldCardTo { player, target, .. } => {
            let (Some(p), Some(t)) = (seat_of(player, seats), seat_of(target, seats)) else {
                return;
            };
            if !ctx.core.player(p).alive || !ctx.core.player(t).alive {
                return;
            }
            for from in seats_from(ctx.core, p) {
                if from == t || !ctx.core.player(from).alive {
                    continue;
                }
                if let Some(c) = ctx.core.player(from).equips.first().copied() {
                    ctx.take_owned_card(from, c);
                    ctx.core.player_mut(t).equips.push(c);
                    ctx.emit(EventData::CardsMoved {
                        cards: vec![c],
                        from,
                        to: t,
                    });
                    break;
                } else if let Some(c) = ctx.core.player(from).judge.first().copied() {
                    ctx.take_owned_card(from, c);
                    ctx.core.player_mut(t).judge.push(c);
                    ctx.emit(EventData::CardsMoved {
                        cards: vec![c],
                        from,
                        to: t,
                    });
                    break;
                }
            }
        }
        EffectRequest::JudgeMakeCurrentShaTargetUnrespondableOnRed {
            player,
            card,
            target,
            retieji,
            ..
        } => {
            let Some(p) = seat_of(player, seats) else {
                return;
            };
            let mut unrespondable = retieji;
            if let Some(jc) = super::commit::pop_deck(ctx) {
                let is_red = matches!(
                    ctx.catalog.cards.get(jc as usize).map(|cd| cd.suit),
                    Some(crate::catalog::Suit::Heart | crate::catalog::Suit::Diamond)
                );
                ctx.to_discard(jc);
                unrespondable = retieji || is_red;
            }
            if unrespondable {
                let cid = card.raw();
                if let Some(t) = seat_of(target, seats) {
                    ctx.core
                        .vm_aux
                        .flags
                        .insert((p, format!("sha_unrespondable:{cid}:{t}")), 1);
                } else {
                    for t in 0..seats {
                        ctx.core
                            .vm_aux
                            .flags
                            .insert((p, format!("sha_unrespondable:{cid}:{t}")), 1);
                    }
                }
            }
        }
        EffectRequest::ChooseMaxMarkOtherJudgeKillUnlessTaoOrTaoyuan { player, mark, .. } => {
            let Some(p) = seat_of(player, seats) else {
                return;
            };
            let mkey = mark.raw();
            let target = seats_from(ctx.core, p)
                .into_iter()
                .filter(|&q| {
                    q != p
                        && ctx.core.player(q).alive
                        && ctx.core.vm_aux.marks.get(&(q, mkey)).copied().unwrap_or(0) > 0
                })
                .max_by_key(|&q| {
                    ctx.core
                        .vm_aux
                        .marks
                        .get(&(q, mkey))
                        .copied()
                        .unwrap_or(0)
                });
            if let Some(t) = target {
                let mut saved = false;
                if let Some(jc) = super::commit::pop_deck(ctx) {
                    saved = matches!(
                        ctx.catalog.card_kind(jc),
                        crate::catalog::CardKind::Tao | crate::catalog::CardKind::Taoyuan
                    );
                    ctx.to_discard(jc);
                }
                if !saved {
                    ctx.emit(EventData::Death {
                        player: t,
                        killer: None,
                    });
                }
            }
        }
        other => {
            super::commit_batch15::commit_batch15_effect(ctx, executing, owner, seats, other);
        }
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

fn scope_candidates(
    core: &GameStateCore,
    p: PlayerId,
    scope: rule_api::DrawTargetScope,
) -> Vec<PlayerId> {
    let min_hand = core
        .players
        .iter()
        .filter(|ps| ps.alive)
        .map(|ps| ps.hand.len())
        .min()
        .unwrap_or(0);
    let my_hand = core.player(p).hand.len();
    let my_hp = core.player(p).hp;
    seats_from(core, p)
        .into_iter()
        .filter(|&q| {
            core.player(q).alive
                && match scope {
                    rule_api::DrawTargetScope::Other => q != p,
                    rule_api::DrawTargetScope::MaleOther => {
                        q != p
                            && core.effective_gender(q)
                                == crate::vocabulary::generals::Gender::Male
                    }
                    rule_api::DrawTargetScope::FemaleOther => {
                        q != p
                            && core.effective_gender(q)
                                == crate::vocabulary::generals::Gender::Female
                    }
                    rule_api::DrawTargetScope::HandLteChooser => {
                        q != p && core.player(q).hand.len() <= my_hand
                    }
                    rule_api::DrawTargetScope::Any => true,
                    rule_api::DrawTargetScope::MinHand => core.player(q).hand.len() == min_hand,
                    rule_api::DrawTargetScope::SameHp => core.player(q).hp == my_hp,
                    rule_api::DrawTargetScope::HandEmpty => core.player(q).hand.is_empty(),
                    rule_api::DrawTargetScope::Wounded => is_wounded(core, q),
                }
        })
        .collect()
}

fn run_fallback_pindian(
    ctx: &mut Ctx<'_>,
    _executing: &str,
    p: PlayerId,
    t: PlayerId,
    selected_card: rule_api::CardId,
    win_skill_def: rule_api::SkillDefId,
    lose_skill_def: rule_api::SkillDefId,
) {
    if p == t
        || !ctx.core.player(p).alive
        || !ctx.core.player(t).alive
        || ctx.core.player(p).hand.is_empty()
        || ctx.core.player(t).hand.is_empty()
    {
        return;
    }
    let pc = if selected_card.is_some()
        && ctx.core.player(p).hand.contains(&selected_card.raw())
    {
        selected_card.raw()
    } else {
        ctx.core.player(p).hand[0]
    };
    let tc = ctx.core.player(t).hand[0];
    let prank = ctx
        .catalog
        .cards
        .get(pc as usize)
        .map(|cd| cd.rank)
        .unwrap_or(0);
    let trank = ctx
        .catalog
        .cards
        .get(tc as usize)
        .map(|cd| cd.rank)
        .unwrap_or(0);
    let win = prank > trank;
    ctx.discard_from_hand(p, pc);
    ctx.discard_from_hand(t, tc);
    if win {
        ctx.core
            .vm_aux
            .flags
            .insert((p, "compare_win_turn".to_string()), 1);
    }
    let grant = if win { win_skill_def } else { lose_skill_def };
    if grant.is_some() {
        if let Some(def) = resolve_skill_def(ctx.catalog, grant.raw()) {
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
    }
}
