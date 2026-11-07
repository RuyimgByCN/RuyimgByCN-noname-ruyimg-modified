//! PA12 批⑯：30 种具名复合结算器（tags 179..=208）的权威状态落账。

use super::commit::{detach, push_pile};
use super::commit_batch15::{
    add_mark_raw, cancel_current_card_use, deal_normal_damage, first_other_alive, get_flag,
    get_mark, grant_skill_raw, is_wounded, lose_hp_raw, modify_current_damage,
    remove_from_current_card_targets, remove_skill_raw, seats_from, set_flag,
};
use super::*;

mod rest;

const ZHENGJIAN_MARK: u32 = 2_446_053_157;

pub(super) fn commit_batch16_effect(
    ctx: &mut Ctx<'_>,
    executing: &str,
    seats: u8,
    e: EffectRequest,
) {
    let _ = executing;
    match e {
        EffectRequest::ResolveStddechong {
            player,
            target,
            cards,
            ..
        } => {
            let Some(p) = seat_of(player, seats) else {
                return;
            };
            let Some(t) = seat_of(target, seats).or_else(|| first_other_alive(ctx.core, p)) else {
                return;
            };
            let chosen: Vec<u32> = if cards.is_empty() {
                ctx.core.player(p).hand.first().copied().into_iter().collect()
            } else {
                cards.iter().map(|c| c.raw()).collect()
            };
            for c in chosen {
                if ctx.core.player(p).hand.contains(&c) {
                    ctx.give_hand_card(p, t, c);
                } else if ctx.core.player(p).equips.contains(&c) {
                    ctx.take_owned_card(p, c);
                    ctx.core.player_mut(t).hand.push(c);
                }
            }
            set_flag(ctx.core, p, "stddechong_target", u32::from(t) + 1);
        }
        EffectRequest::ResolveDddYujin {
            player,
            actor,
            target,
            action,
            ..
        } => {
            let Some(p) = seat_of(player, seats) else {
                return;
            };
            let dst = seat_of(target, seats)
                .or_else(|| seat_of(actor, seats))
                .unwrap_or(p);
            match action {
                0 => {
                    ctx.start_draw(dst, 1);
                    set_flag(ctx.core, p, "dddzhengjun_used", 1);
                }
                1 | 2 => {
                    ctx.start_draw(p, 1);
                    set_flag(ctx.core, p, "dddzhengjun_moved", 1);
                }
                _ => {}
            }
        }
        EffectRequest::ResolveDddZhenji {
            player,
            action,
            cards,
            ..
        } => {
            let Some(p) = seat_of(player, seats) else {
                return;
            };
            match action {
                0 => {
                    let h: Vec<u32> = ctx.core.player(p).hand.iter().copied().take(2).collect();
                    for (i, c) in h.into_iter().enumerate() {
                        ctx.take_hand_card(p, c);
                        let key = if i == 0 {
                            "dddmiaoxing_1"
                        } else {
                            "dddmiaoxing_2"
                        };
                        push_pile(ctx, p, key, c);
                    }
                    set_flag(ctx.core, p, "dddmiaoxing_init", 1);
                }
                1 => {
                    ctx.start_draw(p, 1);
                    set_flag(ctx.core, p, "dddmiaoxing_balanced", 1);
                }
                2 => {
                    let cid = cards
                        .first()
                        .map(|c| c.raw())
                        .or_else(|| {
                            ctx.core
                                .vm_aux
                                .piles
                                .get(&(p, "dddmiaoxing_1".into()))
                                .into_iter()
                                .chain(ctx.core.vm_aux.piles.get(&(p, "dddmiaoxing_2".into())))
                                .flatten()
                                .copied()
                                .next()
                        });
                    if let Some(c) = cid {
                        if detach(ctx.core, c) {
                            ctx.core.player_mut(p).hand.push(c);
                        }
                    }
                    ctx.start_draw(p, 1);
                    set_flag(ctx.core, p, "dddfushi_used", 1);
                }
                _ => {}
            }
        }
        EffectRequest::ResolveZhengjian {
            player,
            stage,
            event_player,
            ..
        } => {
            let Some(p) = seat_of(player, seats) else {
                return;
            };
            match stage {
                0 => {
                    let t = seat_of(event_player, seats)
                        .or_else(|| first_other_alive(ctx.core, p))
                        .unwrap_or(p);
                    if get_mark(ctx.core, t, ZHENGJIAN_MARK) <= 0 {
                        add_mark_raw(ctx.core, t, ZHENGJIAN_MARK, 1);
                    }
                }
                1 | 2 => {
                    if let Some(ep) = seat_of(event_player, seats) {
                        if get_mark(ctx.core, ep, ZHENGJIAN_MARK) > 0 {
                            add_mark_raw(ctx.core, ep, ZHENGJIAN_MARK, 1);
                        }
                    }
                }
                3 => {
                    for q in seats_from(ctx.core, p) {
                        let m = get_mark(ctx.core, q, ZHENGJIAN_MARK);
                        if m > 0 && ctx.core.player(q).alive {
                            let draw_n = (m as u32).min(5).min(ctx.core.player(q).max_hp);
                            ctx.core.vm_aux.marks.remove(&(q, ZHENGJIAN_MARK));
                            if draw_n > 0 {
                                ctx.start_draw(q, draw_n);
                            }
                        }
                    }
                }
                _ => {}
            }
        }
        EffectRequest::ResolveQiaobian {
            player,
            phase,
            stage,
            ..
        } => {
            let Some(p) = seat_of(player, seats) else {
                return;
            };
            if stage == 0 {
                if let Some(&c) = ctx.core.player(p).hand.first() {
                    ctx.discard_from_hand(p, c);
                }
            }
            match phase {
                0 => set_flag(ctx.core, p, "qiaobian_skipped", 1),
                1 => {
                    ctx.core.turn.skip_draw_phase = true;
                    for q in seats_from(ctx.core, p) {
                        if q != p && ctx.core.player(q).alive {
                            if let Some(&c) = ctx.core.player(q).hand.first() {
                                ctx.give_hand_card(q, p, c);
                                break;
                            }
                        }
                    }
                    set_flag(ctx.core, p, "qiaobian_skipped", 2);
                }
                2 => {
                    ctx.core.turn.skip_play_phase = true;
                    set_flag(ctx.core, p, "qiaobian_skipped", 3);
                }
                3 => {
                    ctx.core.player_mut(p).skip_discard_phase = true;
                    set_flag(ctx.core, p, "qiaobian_skipped", 4);
                }
                _ => {}
            }
        }
        EffectRequest::ResolveTwfeiyi {
            player,
            action,
            stage,
            event_cards,
            ..
        } => {
            let Some(p) = seat_of(player, seats) else {
                return;
            };
            match action {
                0 => {
                    set_flag(ctx.core, p, "twshengxi_prepared", 1);
                    ctx.start_draw(p, 1);
                }
                1 => {
                    if stage == 0 {
                        if let Some(idx) = ctx.core.deck.iter().rposition(|&c| {
                            matches!(
                                ctx.catalog.card_kind(c),
                                CardKind::Wuzhong | CardKind::Dismantle | CardKind::Nullify
                            )
                        }) {
                            let c = ctx.core.deck.remove(idx);
                            ctx.core.player_mut(p).hand.push(c);
                            return;
                        }
                    }
                    ctx.start_draw(p, 1);
                }
                2 => {
                    if let Some(t) = first_other_alive(ctx.core, p) {
                        if let Some(&c) = event_cards.first() {
                            if detach(ctx.core, c.raw()) {
                                ctx.core.player_mut(t).hand.push(c.raw());
                                set_flag(ctx.core, p, "twkuanji_used", 1);
                                return;
                            }
                        }
                        ctx.start_draw(t, 1);
                    }
                    set_flag(ctx.core, p, "twkuanji_used", 1);
                }
                _ => {}
            }
        }
        EffectRequest::ResolveZhengsu {
            player,
            mode,
            stage,
            skill,
            ..
        } => {
            let Some(p) = seat_of(player, seats) else {
                return;
            };
            if stage == 0 {
                let n = get_flag(ctx.core, p, "zhengsu_count") + 1;
                set_flag(ctx.core, p, "zhengsu_count", n);
            } else {
                if mode == 2 && is_wounded(ctx.core, p) {
                    ctx.recover(p, 1, None);
                } else {
                    ctx.start_draw(p, 2);
                }
                if skill.is_some() {
                    remove_skill_raw(ctx, p, skill.raw());
                } else if let Some(def) = ctx.catalog.skill_by_name(executing) {
                    ctx.core.skills.remove(&(p, def));
                }
            }
        }
        EffectRequest::ResolveTwcaoang {
            player,
            target,
            card,
            helper_skill_def,
            action,
            ..
        } => {
            let Some(p) = seat_of(player, seats) else {
                return;
            };
            match action {
                0 => {
                    if let Some(t) = seat_of(target, seats) {
                        remove_from_current_card_targets(ctx, card.raw(), t);
                        set_flag(ctx.core, p, "twxiaolian_orig_target", u32::from(t) + 1);
                    }
                    super::commit_batch15::add_to_current_card_targets(ctx, card.raw(), p);
                    set_flag(ctx.core, p, "twxiaolian_redirected", 1);
                }
                1 => {
                    let cid = card
                        .is_some()
                        .then(|| card.raw())
                        .filter(|c| ctx.core.player(p).hand.contains(c))
                        .or_else(|| ctx.core.player(p).hand.first().copied());
                    if let Some(c) = cid {
                        ctx.take_hand_card(p, c);
                        push_pile(ctx, p, "twxiaolian_distance", c);
                    }
                    grant_skill_raw(ctx, p, helper_skill_def, false);
                }
                2 => set_flag(ctx.core, p, "twxiaolian_redirected", 0),
                _ => {}
            }
        }
        EffectRequest::ResolveTwcaozhao {
            player,
            target,
            twfuzuan_skill_def,
            twfeifu_skill_def,
            action,
            ..
        } => {
            let Some(p) = seat_of(player, seats) else {
                return;
            };
            match action {
                0 => {
                    let t = seat_of(target, seats).unwrap_or(p);
                    let cur = get_flag(ctx.core, t, "twfeifu_branch");
                    set_flag(ctx.core, t, "twfeifu_branch", if cur == 0 { 1 } else { 0 });
                    ctx.start_draw(t, 1);
                }
                1 => {
                    grant_skill_raw(ctx, p, twfuzuan_skill_def, false);
                    for q in seats_from(ctx.core, p) {
                        if ctx.core.player(q).alive {
                            grant_skill_raw(ctx, q, twfeifu_skill_def, false);
                        }
                    }
                }
                2 => {
                    let t = seat_of(target, seats)
                        .or_else(|| first_other_alive(ctx.core, p))
                        .unwrap_or(p);
                    if let Some(&c) = ctx.core.player(t).hand.first() {
                        if t != p {
                            ctx.give_hand_card(t, p, c);
                        }
                    }
                    ctx.start_draw(p, 1);
                }
                _ => {}
            }
        }
        EffectRequest::ResolveTwchengpu {
            player,
            target,
            chunlao_skill_def,
            action,
            selected_card,
            ..
        } => {
            let Some(p) = seat_of(player, seats) else {
                return;
            };
            match action {
                0 => {
                    for frame in ctx.res.stack.iter_mut().rev() {
                        if let EventData::CardUsed(u) = &mut frame.data {
                            u.nature = DamageNature::Fire;
                            break;
                        }
                    }
                    set_flag(ctx.core, p, "twlihuo_active", 1);
                }
                1 => {
                    let cid = selected_card
                        .is_some()
                        .then(|| selected_card.raw())
                        .filter(|c| ctx.core.player(p).hand.contains(c))
                        .or_else(|| ctx.core.player(p).hand.first().copied());
                    if let Some(c) = cid {
                        ctx.take_hand_card(p, c);
                        push_pile(ctx, p, "twchunlao", c);
                        grant_skill_raw(ctx, p, chunlao_skill_def, false);
                    }
                }
                2 => {
                    if let Some(pile) = ctx.core.vm_aux.piles.get_mut(&(p, "twchunlao".into())) {
                        if let Some(c) = pile.pop() {
                            ctx.core.discard.push(c);
                        }
                    }
                    modify_current_damage(ctx, 1);
                    set_flag(ctx.core, p, "twchunlao_buff", 1);
                }
                3 => {
                    if let Some(pile) = ctx.core.vm_aux.piles.get_mut(&(p, "twchunlao".into())) {
                        if let Some(c) = pile.pop() {
                            ctx.core.discard.push(c);
                        }
                    }
                    let t = seat_of(target, seats).unwrap_or(p);
                    ctx.recover(t, 1, None);
                }
                _ => {}
            }
        }
        other => commit_batch16_rest(ctx, executing, seats, other),
    }
}

fn commit_batch16_rest(ctx: &mut Ctx<'_>, executing: &str, seats: u8, e: EffectRequest) {
    match e {
        EffectRequest::ResolveTwchenxun {
            player,
            target,
            action,
            ..
        } => {
            let Some(p) = seat_of(player, seats) else {
                return;
            };
            match action {
                0 => {
                    let t = seat_of(target, seats)
                        .filter(|&q| q != p)
                        .or_else(|| first_other_alive(ctx.core, p));
                    if let Some(t) = t {
                        ctx.emit(EventData::CardUsed(crate::event::CardUseData {
                            card: None,
                            player: p,
                            targets: vec![t],
                            nature: DamageNature::Normal,
                            use_as: Some(CardKind::Duel),
                            cancelled: false,
                        }));
                        set_flag(ctx.core, p, "twchenxun_duel", 1);
                    }
                }
                1 => ctx.start_draw(p, 1),
                2 => {
                    lose_hp_raw(ctx, p, 1);
                    set_flag(ctx.core, p, "twchenxun_duel", 0);
                }
                _ => {}
            }
        }
        EffectRequest::ResolveTwchenzhen {
            player,
            target,
            helper_skill_def,
            record_skill_def,
            action,
            card_kind,
            ..
        } => {
            let Some(p) = seat_of(player, seats) else {
                return;
            };
            match action {
                0 => {
                    let want = card_kind_from_tag(card_kind).unwrap_or(CardKind::Sha);
                    if let Some(idx) = ctx
                        .core
                        .deck
                        .iter()
                        .rposition(|&c| ctx.catalog.card_kind(c) == want)
                    {
                        let c = ctx.core.deck.remove(idx);
                        ctx.core.player_mut(p).hand.push(c);
                    } else {
                        ctx.start_draw(p, 1);
                    }
                    grant_skill_raw(ctx, p, helper_skill_def, true);
                }
                1 => {
                    let t = seat_of(target, seats)
                        .filter(|&q| q != p)
                        .or_else(|| first_other_alive(ctx.core, p))
                        .unwrap_or(p);
                    grant_skill_raw(ctx, t, helper_skill_def, false);
                    grant_skill_raw(ctx, t, record_skill_def, false);
                }
                2 => set_flag(ctx.core, p, "twchayi_used", 1),
                3 => {
                    ctx.start_draw(p, 1);
                    remove_skill_raw(ctx, p, helper_skill_def);
                }
                _ => {}
            }
        }
        EffectRequest::ResolveTwyizhu {
            player,
            action,
            ..
        } => {
            let Some(p) = seat_of(player, seats) else {
                return;
            };
            match action {
                0 => {
                    ctx.start_draw(p, 1);
                    set_flag(ctx.core, p, "twyizhu_tracked", 1);
                }
                1 => {
                    remove_from_current_card_targets(ctx, 0, p);
                    ctx.start_draw(p, 1);
                }
                2 => set_flag(ctx.core, p, "twyizhu_tracked", 0),
                _ => {}
            }
        }
        other => rest::commit_batch16_rest(ctx, executing, seats, other),
    }
}

