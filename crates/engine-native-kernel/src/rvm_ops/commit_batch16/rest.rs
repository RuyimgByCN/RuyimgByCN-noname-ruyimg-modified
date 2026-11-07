//! PA12 批⑯后半部分（tags 192..=208）的权威状态落账。

use super::*;

const POTTUNTIAN_MARK: u32 = 2_435_155_097;
const JILUE_JIZHI_MARK: u32 = 2_747_240_883;
const JILUE_WANSHA_MARK: u32 = 3_522_766_701;

pub(super) fn commit_batch16_rest(
    ctx: &mut Ctx<'_>,
    executing: &str,
    seats: u8,
    e: EffectRequest,
) {
    match e {
        EffectRequest::ResolveRuanji {
            player,
            card,
            action,
            cards,
            ..
        } => {
            let Some(p) = seat_of(player, seats) else {
                return;
            };
            match action {
                0 => {
                    let hand = ctx.core.player(p).hand.clone();
                    for c in hand {
                        set_flag(ctx.core, p, &format!("dczhaowen_card:{c}"), 1);
                    }
                    set_flag(ctx.core, p, "dczhaowen_active", 1);
                }
                1 => ctx.start_draw(p, 1),
                2 => {
                    cancel_current_card_use(ctx, card.raw());
                    set_flag(ctx.core, p, "dcjiudun_sha_buff", 1);
                }
                3 => {
                    if get_flag(ctx.core, p, "dcjiudun_drunk") == 0 {
                        set_flag(ctx.core, p, "dcjiudun_drunk", 1);
                        ctx.start_draw(p, 1);
                    } else {
                        let cid = cards
                            .first()
                            .map(|c| c.raw())
                            .filter(|c| ctx.core.player(p).hand.contains(c))
                            .or_else(|| ctx.core.player(p).hand.first().copied());
                        if let Some(c) = cid {
                            ctx.discard_from_hand(p, c);
                        }
                        cancel_current_card_use(ctx, card.raw());
                    }
                }
                _ => {}
            }
        }
        EffectRequest::ResolveQiaojian {
            player,
            target,
            helper_def,
            action,
            cards,
            ..
        } => {
            let Some(p) = seat_of(player, seats) else {
                return;
            };
            match action {
                0 => {
                    let chosen: Vec<u32> = if cards.is_empty() {
                        ctx.core.player(p).hand.first().copied().into_iter().collect()
                    } else {
                        cards.iter().map(|c| c.raw()).collect()
                    };
                    for c in chosen {
                        if ctx.core.player(p).hand.contains(&c) {
                            ctx.discard_from_hand(p, c);
                        }
                    }
                    if let Some(t) = seat_of(target, seats).or_else(|| first_other_alive(ctx.core, p)) {
                        if let Some(&tc) = ctx.core.player(t).hand.first() {
                            ctx.discard_from_hand(t, tc);
                        }
                        grant_skill_raw(ctx, t, helper_def.raw(), false);
                    }
                    if is_wounded(ctx.core, p) {
                        ctx.recover(p, 1, None);
                    } else {
                        ctx.start_draw(p, 1);
                    }
                }
                5 => {
                    set_flag(ctx.core, p, "qiaojian_extra_ready", 1);
                    remove_skill_raw(ctx, p, helper_def.raw());
                }
                6 => {
                    ctx.start_draw(p, 1);
                    set_flag(ctx.core, p, "qiaojian_extra_ready", 0);
                }
                9 => set_flag(ctx.core, p, "qiaojian_tracked", 0),
                _ => {}
            }
        }
        EffectRequest::ResolveZhupeilan {
            player,
            target,
            helper_skill_def,
            action,
            ..
        } => {
            let Some(p) = seat_of(player, seats) else {
                return;
            };
            match action {
                0 => {
                    grant_skill_raw(ctx, p, helper_skill_def.raw(), true);
                    set_flag(ctx.core, p, "dccilv_tracked", 1);
                }
                1 => {
                    modify_current_damage(ctx, -1);
                    ctx.start_draw(p, 1);
                }
                2 => {
                    remove_skill_raw(ctx, p, helper_skill_def.raw());
                    set_flag(ctx.core, p, "dccilv_tracked", 0);
                }
                10..=12 => {
                    ctx.recover(p, 1, None);
                    if let Some(t) = seat_of(target, seats).or_else(|| first_other_alive(ctx.core, p)) {
                        ctx.start_draw(t, 1);
                    }
                    set_flag(ctx.core, p, "dctongdao_used", 1);
                }
                _ => {}
            }
        }
        EffectRequest::ResolveMbduanyang {
            player,
            card,
            mode,
            cards,
            ..
        } => {
            let Some(p) = seat_of(player, seats) else {
                return;
            };
            match mode {
                0 => {
                    let cid = cards
                        .first()
                        .map(|c| c.raw())
                        .or_else(|| card.is_some().then(|| card.raw()))
                        .or_else(|| ctx.core.player(p).hand.first().copied());
                    if let Some(c) = cid {
                        if detach(ctx.core, c) {
                            push_pile(ctx, p, "mbduanyang", c);
                        }
                    }
                    set_flag(ctx.core, p, "mbduanyang_stored", 1);
                }
                1 => {
                    let piled = ctx
                        .core
                        .vm_aux
                        .piles
                        .remove(&(p, "mbduanyang".into()))
                        .unwrap_or_default();
                    if piled.is_empty() {
                        ctx.start_draw(p, 1);
                    } else {
                        ctx.core.player_mut(p).hand.extend(piled);
                    }
                }
                2 => ctx.start_draw(p, 1),
                _ => {}
            }
        }
        EffectRequest::ResolveMbrunwei {
            player,
            target,
            stage,
            ..
        } => {
            let Some(p) = seat_of(player, seats) else {
                return;
            };
            match stage {
                0 => {
                    let t = seat_of(target, seats).unwrap_or(p);
                    ctx.start_draw(t, 1);
                    set_flag(ctx.core, p, "mbrunwei_active", 1);
                }
                4 => ctx.start_draw(p, 1),
                5 => {
                    set_flag(ctx.core, p, "mbrunwei_active", 0);
                    ctx.start_draw(p, 1);
                }
                _ => {}
            }
        }
        EffectRequest::ResolveMbkuangxiang {
            player,
            target,
            stage,
            ..
        } => {
            let Some(p) = seat_of(player, seats) else {
                return;
            };
            match stage {
                0 => {
                    let t = seat_of(target, seats)
                        .filter(|&q| q != p)
                        .or_else(|| first_other_alive(ctx.core, p))
                        .unwrap_or(p);
                    ctx.start_draw(t, 2);
                    set_flag(ctx.core, p, "mbkuangxiang_target", u32::from(t) + 1);
                }
                1 => {
                    if is_wounded(ctx.core, p) {
                        ctx.recover(p, 1, None);
                    }
                    ctx.start_draw(p, 1);
                }
                2 => set_flag(ctx.core, p, "mbkuangxiang_target", 0),
                _ => {}
            }
        }
        EffectRequest::ResolvePotsifeng {
            player,
            target,
            card,
            stage,
            ..
        } => {
            let Some(p) = seat_of(player, seats) else {
                return;
            };
            match stage {
                0 => {
                    let cid = card
                        .is_some()
                        .then(|| card.raw())
                        .filter(|c| ctx.core.player(p).hand.contains(c))
                        .or_else(|| ctx.core.player(p).hand.first().copied());
                    if let Some(c) = cid {
                        ctx.take_hand_card(p, c);
                        push_pile(ctx, p, "potsifeng", c);
                    }
                    set_flag(ctx.core, p, "potsifeng_active", 1);
                }
                3 => ctx.start_draw(p, 1),
                4 => {
                    let piled = ctx
                        .core
                        .vm_aux
                        .piles
                        .remove(&(p, "potsifeng".into()))
                        .unwrap_or_default();
                    ctx.core.discard.extend(piled);
                    if let Some(t) = seat_of(target, seats).filter(|&q| q != p && ctx.core.player(q).alive) {
                        deal_normal_damage(ctx, Some(p), t, 1);
                    } else {
                        ctx.start_draw(p, 1);
                    }
                }
                _ => {}
            }
        }
        EffectRequest::ResolvePotwanglie {
            player,
            card,
            helper_skill_def,
            stage,
            ..
        } => {
            let Some(p) = seat_of(player, seats) else {
                return;
            };
            match stage {
                0 => {
                    set_flag(ctx.core, p, "potwanglie_armed", 1);
                    if card.is_some() {
                        set_flag(ctx.core, p, "potwanglie_card", card.raw());
                    }
                }
                1 => {
                    grant_skill_raw(ctx, p, helper_skill_def.raw(), true);
                    ctx.start_draw(p, 1);
                }
                2 => {
                    set_flag(ctx.core, p, "potwanglie_armed", 0);
                    remove_skill_raw(ctx, p, helper_skill_def.raw());
                }
                _ => {}
            }
        }
        EffectRequest::ResolvePotchengpu {
            player,
            target,
            stage,
            ..
        } => {
            let Some(p) = seat_of(player, seats) else {
                return;
            };
            match stage {
                0 | 1 => {
                    if let Some(t) = seat_of(target, seats)
                        .filter(|&q| q != p)
                        .or_else(|| first_other_alive(ctx.core, p))
                    {
                        if let Some(&c) = ctx.core.player(p).hand.first() {
                            ctx.give_hand_card(p, t, c);
                        } else {
                            ctx.start_draw(t, 1);
                        }
                    }
                    set_flag(ctx.core, p, "potduzuo_used", 1);
                }
                2 | 3 => {
                    modify_current_damage(ctx, -99);
                    ctx.start_draw(p, 2);
                    set_flag(ctx.core, p, "potbihan_used", 1);
                }
                _ => {}
            }
        }
        EffectRequest::ResolvePottuntian {
            player,
            action,
            amount,
            targets,
            zaoxian_def,
            ..
        } => {
            let Some(p) = seat_of(player, seats) else {
                return;
            };
            match action {
                0 => add_mark_raw(ctx.core, p, POTTUNTIAN_MARK, 1),
                1 => {
                    set_flag(ctx.core, p, "pottuntian_cap", 4);
                    if get_mark(ctx.core, p, POTTUNTIAN_MARK) >= 3 {
                        grant_skill_raw(ctx, p, zaoxian_def.raw(), false);
                    }
                }
                2 | 3 => {
                    let spend = i32::from(amount.max(1));
                    add_mark_raw(ctx.core, p, POTTUNTIAN_MARK, -spend);
                    if targets.is_empty() {
                        ctx.start_draw(p, 1);
                    } else {
                        for tp in targets {
                            if let Some(q) = seat_of(tp, seats) {
                                ctx.start_draw(q, 1);
                            }
                        }
                    }
                }
                _ => {}
            }
        }
        EffectRequest::ResolvePotjixi {
            player,
            target,
            action,
            targets,
            ..
        } => {
            let Some(p) = seat_of(player, seats) else {
                return;
            };
            match action {
                0 => set_flag(ctx.core, p, "potjixi_targets", 0),
                1 => {
                    if let Some(t) = seat_of(target, seats) {
                        let m = get_flag(ctx.core, p, "potjixi_targets") | (1u32 << t);
                        set_flag(ctx.core, p, "potjixi_targets", m);
                    }
                }
                2..=4 => {
                    let t = targets
                        .first()
                        .and_then(|tp| seat_of(*tp, seats))
                        .or_else(|| first_other_alive(ctx.core, p));
                    if let Some(t) = t {
                        if let Some(&c) = ctx.core.player(t).hand.first() {
                            ctx.give_hand_card(t, p, c);
                            return;
                        }
                    }
                    ctx.start_draw(p, 1);
                }
                _ => {}
            }
        }
        EffectRequest::ResolvePotjiyu {
            player,
            card,
            stage,
            ..
        } => {
            let Some(p) = seat_of(player, seats) else {
                return;
            };
            match stage {
                0 => {
                    let cid = card
                        .is_some()
                        .then(|| card.raw())
                        .filter(|c| ctx.core.player(p).hand.contains(c))
                        .or_else(|| ctx.core.player(p).hand.first().copied());
                    if let Some(c) = cid {
                        ctx.discard_from_hand(p, c);
                    }
                    ctx.start_draw(p, 1);
                    set_flag(ctx.core, p, "potjiyu_active", 1);
                }
                1 => set_flag(ctx.core, p, "potjiyu_moved", 1),
                2 => {
                    if get_flag(ctx.core, p, "potjiyu_active") != 0 {
                        ctx.start_draw(p, 1);
                        set_flag(ctx.core, p, "potjiyu_active", 0);
                    }
                }
                _ => {}
            }
        }
        EffectRequest::ResolvePotnigu {
            player,
            stage,
            ..
        } => {
            let Some(p) = seat_of(player, seats) else {
                return;
            };
            match stage {
                0..=2 => {
                    let mut declined = 0u32;
                    for q in seats_from(ctx.core, p) {
                        if q != p && ctx.core.player(q).alive {
                            if let Some(&c) = ctx.core.player(q).hand.first() {
                                ctx.give_hand_card(q, p, c);
                            } else {
                                declined += 1;
                            }
                        }
                    }
                    set_flag(ctx.core, p, "potnigu_declined", declined);
                    if declined == 0 {
                        ctx.start_draw(p, 1);
                    }
                }
                3 => {
                    if get_flag(ctx.core, p, "potnigu_declined") > 0 {
                        modify_current_damage(ctx, 1);
                    }
                }
                4 => set_flag(ctx.core, p, "potnigu_declined", 0),
                _ => {}
            }
        }
        EffectRequest::ResolvePotfuan {
            player,
            stage,
            range_helper_def,
            yinxian_def,
            ..
        } => {
            let Some(p) = seat_of(player, seats) else {
                return;
            };
            match stage {
                0 => {
                    grant_skill_raw(ctx, p, range_helper_def.raw(), false);
                    set_flag(ctx.core, p, "potfuan_suit", 1);
                    ctx.start_draw(p, 1);
                }
                2 => {
                    remove_skill_raw(ctx, p, range_helper_def.raw());
                    grant_skill_raw(ctx, p, yinxian_def.raw(), false);
                    ctx.start_draw(p, 2);
                }
                3 => {
                    remove_skill_raw(ctx, p, range_helper_def.raw());
                    set_flag(ctx.core, p, "potfuan_suit", 0);
                }
                _ => {}
            }
        }
        EffectRequest::ResolveDddGuanning {
            player,
            action,
            card,
            ..
        } => {
            let Some(p) = seat_of(player, seats) else {
                return;
            };
            match action {
                0 => {
                    set_flag(ctx.core, p, "ddd_youxue_ready", 1);
                    ctx.start_draw(p, 1);
                }
                1 => {
                    ctx.start_draw(p, 1);
                    set_flag(ctx.core, p, "ddd_chengjing_end", 1);
                }
                2 => {
                    let cid = card
                        .is_some()
                        .then(|| card.raw())
                        .filter(|c| ctx.core.player(p).hand.contains(c))
                        .or_else(|| ctx.core.player(p).hand.first().copied());
                    if let Some(c) = cid {
                        ctx.discard_from_hand(p, c);
                    }
                    ctx.start_draw(p, 2);
                }
                _ => {}
            }
        }
        EffectRequest::ResolveDddJianshuo {
            player,
            target,
            action,
            cards,
            ..
        } => {
            let Some(p) = seat_of(player, seats) else {
                return;
            };
            match action {
                0 => {
                    let cid = cards
                        .first()
                        .map(|c| c.raw())
                        .filter(|c| ctx.core.player(p).hand.contains(c))
                        .or_else(|| ctx.core.player(p).hand.first().copied());
                    if let Some(c) = cid {
                        ctx.discard_from_hand(p, c);
                    }
                    if let Some(t) = seat_of(target, seats).or_else(|| first_other_alive(ctx.core, p)) {
                        if let Some(&tc) = ctx.core.player(t).hand.first() {
                            ctx.discard_from_hand(t, tc);
                        }
                    }
                    ctx.start_draw(p, 1);
                    set_flag(ctx.core, p, "dddfenye_used", 1);
                }
                1 => {
                    ctx.start_draw(p, 2);
                    set_flag(ctx.core, p, "dddshichao_prep", 1);
                }
                2 => {
                    modify_current_damage(ctx, 1);
                    ctx.start_draw(p, 1);
                }
                _ => {}
            }
        }
        EffectRequest::ResolveShenSimayi {
            player,
            event_amount,
            mark,
            mode,
            cards,
            ..
        } => {
            let Some(p) = seat_of(player, seats) else {
                return;
            };
            match mode {
                0 => {
                    let gain = (event_amount.max(1)) as i32;
                    add_mark_raw(ctx.core, p, mark.raw(), gain);
                }
                1 => {
                    set_flag(ctx.core, p, "lianpo_extra_turn", 1);
                    ctx.core.turn.extra_phases.extend([2, 3]);
                    ctx.start_draw(p, 1);
                }
                2 => {
                    add_mark_raw(ctx.core, p, mark.raw(), -1);
                    add_mark_raw(ctx.core, p, JILUE_WANSHA_MARK, 1);
                }
                3 => {
                    add_mark_raw(ctx.core, p, mark.raw(), -1);
                    let chosen: Vec<u32> = if cards.is_empty() {
                        ctx.core.player(p).hand.first().copied().into_iter().collect()
                    } else {
                        cards.iter().map(|c| c.raw()).collect()
                    };
                    let mut count = 0u32;
                    for c in chosen {
                        if ctx.core.player(p).hand.contains(&c) {
                            ctx.discard_from_hand(p, c);
                            count += 1;
                        } else if ctx.core.player(p).equips.contains(&c) {
                            ctx.take_owned_card(p, c);
                            ctx.core.discard.push(c);
                            count += 1;
                        }
                    }
                    ctx.start_draw(p, count.max(1));
                }
                4 => {
                    add_mark_raw(ctx.core, p, mark.raw(), -1);
                    add_mark_raw(ctx.core, p, JILUE_JIZHI_MARK, 1);
                    ctx.start_draw(p, 1);
                }
                _ => {}
            }
        }
        other => super::super::commit_batch17::commit_batch17_effect(ctx, executing, seats, other),
    }
}
