//! PA12 批⑮（上半部分）：tags 154..=163 的权威落账与共享辅助函数。

use super::commit::detach;
use super::*;

const POTWEIYAN_KUANGGU_DEF: u32 = 4_200_000_562;
const POTWEIYAN_ZHUANGSHI_DEF: u32 = 4_200_000_560;
const POTWEIYAN_KUNFEN_DEF: u32 = 3_536_332_483;
const POTWEIYAN_DIRECT_DEF: u32 = 4_200_000_563;
const POTWEIYAN_LIMIT_DEF: u32 = 4_200_000_564;
const POTWEIYAN_KUANGGU_LIMIT_DEF: u32 = 4_200_000_565;
const POTNIZUN_DEF: u32 = 4_200_000_576;
const POTFUJI_EFFECT_DEF: u32 = 4_200_000_579;
const POTFUJI_SHA_DEF: u32 = 4_200_000_580;
const POTFUJI_SHAN_DEF: u32 = 4_200_000_581;
pub(super) const DDD_FENGZHENG_GLOBAL_DEF: u32 = 1_253_882_292;
pub(super) const DDD_BAILEI_DEF: u32 = 2_689_073_031;

pub(super) fn commit_batch15_effect(
    ctx: &mut Ctx<'_>,
    executing: &str,
    owner: PlayerId,
    seats: u8,
    e: EffectRequest,
) {
    let _ = owner;
    match e {
        EffectRequest::ResolvePotweiyan {
            player,
            target,
            card,
            action,
            skill,
        } => {
            let Some(p) = seat_of(player, seats) else {
                return;
            };
            match action {
                0 => {
                    if get_flag(ctx.core, p, "potweiyan_status") == 0 {
                        grant_skill_raw(ctx, p, POTWEIYAN_KUANGGU_DEF, false);
                    }
                }
                1 => {
                    if get_flag(ctx.core, p, "potweiyan_status") == 0 {
                        set_flag(ctx.core, p, "potweiyan_status", 1);
                        set_flag(ctx.core, p, "potkuanggu_upgraded", 1);
                        let direct_left = get_mark(ctx.core, p, POTWEIYAN_DIRECT_DEF);
                        let limit_left = get_mark(ctx.core, p, POTWEIYAN_LIMIT_DEF);
                        if limit_left > 0 {
                            ctx.start_draw(p, 1);
                        }
                        if direct_left > 0 {
                            if is_wounded(ctx.core, p) {
                                ctx.recover(p, 1, None);
                            } else {
                                ctx.start_draw(p, 1);
                            }
                        }
                    }
                }
                2 => {
                    let cid = card
                        .is_some()
                        .then(|| card.raw())
                        .filter(|c| ctx.core.player(p).hand.contains(c))
                        .or_else(|| ctx.core.player(p).hand.first().copied());
                    if let Some(c) = cid {
                        ctx.discard_from_hand(p, c);
                        add_mark_raw(ctx.core, p, POTWEIYAN_DIRECT_DEF, 1);
                        grant_skill_raw(ctx, p, POTWEIYAN_DIRECT_DEF, false);
                    } else {
                        lose_hp_raw(ctx, p, 1);
                        add_mark_raw(ctx.core, p, POTWEIYAN_LIMIT_DEF, 1);
                        grant_skill_raw(ctx, p, POTWEIYAN_LIMIT_DEF, false);
                    }
                }
                3 => {
                    set_flag(ctx.core, p, "potweiyan_status", 2);
                    remove_skill_raw(ctx, p, POTWEIYAN_ZHUANGSHI_DEF);
                    grant_skill_raw(ctx, p, POTWEIYAN_KUNFEN_DEF, false);
                }
                4 | 5 => {
                    let def = if action == 4 {
                        POTWEIYAN_DIRECT_DEF
                    } else {
                        POTWEIYAN_LIMIT_DEF
                    };
                    add_mark_raw(ctx.core, p, def, -1);
                    if get_mark(ctx.core, p, def) <= 0 {
                        remove_skill_raw(ctx, p, def);
                    }
                }
                6 => remove_skill_raw(ctx, p, skill.raw()),
                7 => {
                    if let Some(t) = seat_of(target, seats) {
                        if ctx.core.player(p).hp <= ctx.core.player(t).hp {
                            modify_current_damage(ctx, 1);
                        }
                        if ctx.core.player(p).hand.len() <= ctx.core.player(t).hand.len() {
                            set_flag(ctx.core, p, "potyinzhan_target", t as u32 + 1);
                            set_flag(ctx.core, p, "potyinzhan_card", card.raw());
                        }
                    }
                }
                8 | 11 => {
                    let t = seat_of(target, seats).or_else(|| {
                        let raw = get_flag(ctx.core, p, "potyinzhan_target");
                        (raw > 0).then(|| (raw - 1) as u8)
                    });
                    set_flag(ctx.core, p, "potyinzhan_target", 0);
                    if let Some(t) = t {
                        if !ctx.core.player(t).hand.is_empty() {
                            let c = ctx.core.player(t).hand[0];
                            ctx.discard_from_hand(t, c);
                        } else {
                            ctx.start_draw(p, 1);
                        }
                    }
                }
                9 => {
                    if is_wounded(ctx.core, p) {
                        ctx.recover(p, 1, None);
                    } else {
                        ctx.start_draw(p, 1);
                    }
                }
                10 => {
                    lose_hp_raw(ctx, p, 1);
                    ctx.start_draw(p, 2);
                }
                12 | 13 => {
                    if action == 12 {
                        let cid = card
                            .is_some()
                            .then(|| card.raw())
                            .filter(|c| ctx.core.player(p).hand.contains(c))
                            .or_else(|| ctx.core.player(p).hand.first().copied());
                        if let Some(c) = cid {
                            ctx.discard_from_hand(p, c);
                        }
                    }
                    add_mark_raw(ctx.core, p, POTWEIYAN_KUANGGU_LIMIT_DEF, 1);
                    grant_skill_raw(ctx, p, POTWEIYAN_KUANGGU_LIMIT_DEF, false);
                }
                14 | 15 => {
                    lose_hp_raw(ctx, p, 1);
                    add_mark_raw(ctx.core, p, POTWEIYAN_LIMIT_DEF, 1);
                    grant_skill_raw(ctx, p, POTWEIYAN_LIMIT_DEF, false);
                }
                _ => {}
            }
        }
        EffectRequest::ResolvePotcaoshuang {
            player,
            action,
            ..
        } => {
            let Some(p) = seat_of(player, seats) else {
                return;
            };
            match action {
                0..=2 => {
                    let bit = 1u32 << action;
                    let cur = get_flag(ctx.core, p, "potcaoshuang_assigned");
                    if cur == 0b111 {
                        set_flag(ctx.core, p, "potjianzhuan_awakened", 1);
                        grant_skill_raw(ctx, p, POTNIZUN_DEF, false);
                    } else {
                        set_flag(ctx.core, p, "potcaoshuang_assigned", cur | bit);
                    }
                }
                3..=5 => {
                    let bit = 1u32 << (action - 3);
                    let cur = get_flag(ctx.core, p, "potdianyi_triggered");
                    set_flag(ctx.core, p, "potdianyi_triggered", cur | bit);
                    let want = (cur.count_ones() + 1) as usize;
                    let tagged: Vec<CardId> = ctx
                        .core
                        .discard
                        .iter()
                        .chain(ctx.core.deck.iter())
                        .copied()
                        .filter(|c| get_flag(ctx.core, 0, &format!("shequan_card:{c}")) != 0)
                        .take(want)
                        .collect();
                    let mut gained = 0u32;
                    for c in tagged {
                        if detach(ctx.core, c) {
                            ctx.core.player_mut(p).hand.push(c);
                            gained += 1;
                        }
                    }
                    if gained == 0 {
                        ctx.start_draw(p, 1);
                        gained = 1;
                    }
                    let total = get_flag(ctx.core, p, "potdianyi_gained") + gained;
                    set_flag(ctx.core, p, "potdianyi_gained", total);
                    if total > ctx.core.player(p).max_hp as u32 {
                        lose_hp_raw(ctx, p, 1);
                    }
                }
                6..=8 => {
                    let bit = 1u32 << (action - 6);
                    let cur = get_flag(ctx.core, p, "potshequan_triggered");
                    set_flag(ctx.core, p, "potshequan_triggered", cur | bit);
                    for q in seats_from(ctx.core, p) {
                        if q != p && ctx.core.player(q).alive {
                            if let Some(&c) = ctx
                                .core
                                .player(q)
                                .hand
                                .iter()
                                .find(|&&c| get_flag(ctx.core, 0, &format!("shequan_card:{c}")) == 0)
                            {
                                set_flag(ctx.core, 0, &format!("shequan_card:{c}"), 1);
                            }
                        }
                    }
                }
                9..=12 => {
                    let tagged: Vec<CardId> = ctx
                        .core
                        .discard
                        .iter()
                        .copied()
                        .filter(|c| get_flag(ctx.core, 0, &format!("shequan_card:{c}")) != 0)
                        .collect();
                    for c in tagged {
                        if detach(ctx.core, c) {
                            ctx.core.player_mut(p).hand.push(c);
                        }
                    }
                }
                _ => {}
            }
        }
        EffectRequest::ResolvePotlvyi {
            player,
            target,
            card,
            action,
            event_cards,
            ..
        } => {
            let Some(p) = seat_of(player, seats) else {
                return;
            };
            match action {
                0 => {
                    let colors = get_flag(ctx.core, p, "pothuilv_colors");
                    set_flag(ctx.core, p, "pothuilv_colors", colors | 1);
                    if let Some(t) = seat_of(target, seats) {
                        if let Some(&c) = ctx.core.player(t).hand.first() {
                            set_flag(ctx.core, p, "pothuilv_actor", t as u32 + 1);
                            set_flag(ctx.core, p, "pothuilv_card", c);
                            set_flag(ctx.core, p, "pothuilv_used", 0);
                        }
                    }
                }
                1 => {
                    if card.is_some() && get_flag(ctx.core, p, "pothuilv_card") == card.raw() {
                        set_flag(ctx.core, p, "pothuilv_used", 1);
                    }
                }
                2 => {
                    let tracked = get_flag(ctx.core, p, "pothuilv_card");
                    let used = get_flag(ctx.core, p, "pothuilv_used") != 0;
                    set_flag(ctx.core, p, "pothuilv_card", 0);
                    set_flag(ctx.core, p, "pothuilv_used", 0);
                    let Some(t) = seat_of(target, seats) else {
                        return;
                    };
                    if used && ctx.core.player(t).alive {
                        deal_normal_damage(ctx, Some(p), t, 1);
                    }
                    if tracked != 0 && !ctx.core.discard.contains(&tracked) && ctx.core.player(p).alive {
                        deal_normal_damage(ctx, Some(t), p, 1);
                    }
                }
                3 => {
                    let mut mask = get_flag(ctx.core, p, "potsongyan_colors");
                    for c in event_cards {
                        if let Some(cd) = ctx.catalog.cards.get(c.raw() as usize) {
                            mask |= if matches!(cd.suit, Suit::Heart | Suit::Diamond) {
                                1
                            } else {
                                2
                            };
                        }
                    }
                    set_flag(ctx.core, p, "potsongyan_colors", mask);
                }
                4 => {
                    if ctx.core.player(p).alive {
                        ctx.emit(EventData::CardUsed(crate::event::CardUseData {
                            card: None,
                            player: p,
                            targets: vec![p],
                            nature: DamageNature::Normal,
                            use_as: Some(CardKind::Wuzhong),
                            cancelled: false,
                        }));
                    }
                }
                5 | 9 => {
                    set_flag(ctx.core, p, "potsongyan_card", 0);
                    set_flag(ctx.core, p, "potshishi_card", 0);
                }
                6 => {
                    if let Some(t) = seat_of(target, seats) {
                        if t != p && get_flag(ctx.core, p, "potshishi_disabled") == 0 {
                            set_flag(ctx.core, p, &format!("potshishi_src:{t}"), 1);
                        }
                    }
                }
                7 => {
                    if card.is_some() && get_flag(ctx.core, p, "potshishi_disabled") == 0 {
                        set_flag(ctx.core, p, "potshishi_card", card.raw());
                        for q in 0..seats {
                            if get_flag(ctx.core, p, &format!("potshishi_src:{q}")) != 0 {
                                set_flag(
                                    ctx.core,
                                    p,
                                    &format!("sha_unrespondable:{}:{q}", card.raw()),
                                    1,
                                );
                            }
                        }
                    }
                }
                8 => {
                    if seat_of(target, seats).is_some_and(|t| t != p) {
                        set_flag(ctx.core, p, "potshishi_disabled", 1);
                    }
                }
                10 | 11 => {
                    let cid = card.raw();
                    if cid != 0 && !ctx.core.discard.contains(&cid) {
                        let avoid = seat_of(target, seats);
                        if let Some(dst) = seats_from(ctx.core, p)
                            .into_iter()
                            .find(|&q| Some(q) != avoid && ctx.core.player(q).alive)
                        {
                            if detach(ctx.core, cid) {
                                ctx.core.player_mut(dst).hand.push(cid);
                            }
                        }
                    }
                }
                _ => {}
            }
        }
        EffectRequest::ResolvePotyuji {
            player,
            card,
            action,
            cards,
            targets,
            ..
        } => {
            let Some(p) = seat_of(player, seats) else {
                return;
            };
            match action {
                0 => {
                    for (c, dst_id) in cards.iter().zip(targets.iter()) {
                        let Some(dst) = seat_of(*dst_id, seats) else {
                            continue;
                        };
                        let cid = c.raw();
                        set_flag(ctx.core, 0, &format!("potfuji_card:{cid}"), 1);
                        if ctx.core.player(p).hand.contains(&cid) {
                            ctx.give_hand_card(p, dst, cid);
                        } else if detach(ctx.core, cid) {
                            ctx.core.player_mut(dst).hand.push(cid);
                        }
                        grant_skill_raw(ctx, dst, POTFUJI_EFFECT_DEF, false);
                    }
                    let min_hand = ctx
                        .core
                        .players
                        .iter()
                        .filter(|ps| ps.alive)
                        .map(|ps| ps.hand.len())
                        .min()
                        .unwrap_or(0);
                    if ctx.core.player(p).hand.len() == min_hand {
                        ctx.start_draw(p, 2);
                        grant_skill_raw(ctx, p, POTFUJI_SHA_DEF, false);
                        grant_skill_raw(ctx, p, POTFUJI_SHAN_DEF, false);
                    }
                }
                1 | 4 | 5 => {
                    gain_same_suit_or_draw(ctx, p, card.raw());
                    if action == 4 && card.is_some() {
                        *ctx.core
                            .vm_aux
                            .flags
                            .entry((p, format!("card_damage_bonus:{}", card.raw())))
                            .or_insert(0) += 1;
                    }
                }
                2 => modify_current_damage(ctx, 1),
                3 => {
                    ctx.start_draw(p, 1);
                    if card.is_some() {
                        set_flag(ctx.core, 0, &format!("potfuji_card:{}", card.raw()), 0);
                    }
                    for c in cards {
                        set_flag(ctx.core, 0, &format!("potfuji_card:{}", c.raw()), 0);
                    }
                }
                6 => remove_skill_raw(ctx, p, POTFUJI_SHA_DEF),
                7 => {
                    ctx.start_draw(p, 1);
                    remove_skill_raw(ctx, p, POTFUJI_SHAN_DEF);
                }
                8 => {
                    remove_skill_raw(ctx, p, POTFUJI_SHA_DEF);
                    remove_skill_raw(ctx, p, POTFUJI_SHAN_DEF);
                }
                _ => {}
            }
        }
        EffectRequest::ResolvePotzhouyu {
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
                0 => {
                    if !ctx.core.player(p).hand.is_empty() {
                        set_flag(ctx.core, p, "potyanhui_armed", 1);
                        let rev = get_flag(ctx.core, p, "potyanhui_revealed");
                        set_flag(ctx.core, p, "potyanhui_revealed", rev + 1);
                    }
                }
                1 => {
                    for tid in targets {
                        if let Some(t) = seat_of(tid, seats) {
                            if ctx.core.player(t).alive && !ctx.core.player(t).hand.is_empty() {
                                let c = ctx.core.player(t).hand[0];
                                ctx.discard_from_hand(t, c);
                                set_flag(ctx.core, p, "potyanhui_discarded_target", t as u32 + 1);
                                break;
                            }
                        }
                    }
                }
                2 => {
                    if get_flag(ctx.core, p, "potyanhui_armed") != 0 {
                        set_flag(ctx.core, p, "potyanhui_armed", 0);
                        let rev = get_flag(ctx.core, p, "potyanhui_revealed").max(1);
                        set_flag(ctx.core, p, "potyanhui_revealed", 0);
                        ctx.start_draw(p, rev);
                    }
                }
                3 => {
                    if let Some(t) = seat_of(target, seats) {
                        if t != p && ctx.core.player(t).alive && ctx.core.player(t).linked {
                            modify_current_damage(ctx, 1);
                        }
                    }
                }
                4 => {
                    if let Some(t) = seat_of(target, seats) {
                        if ctx.core.player(t).alive && !ctx.core.player(t).linked {
                            ctx.core.player_mut(t).linked = true;
                            ctx.emit(EventData::Linked {
                                player: t,
                                linked: true,
                            });
                        }
                    }
                }
                5 => {
                    set_flag(ctx.core, p, "potyanhui_armed", 0);
                    set_flag(ctx.core, p, "potyanhui_revealed", 0);
                }
                6..=10 => {
                    ctx.start_draw(p, 1);
                }
                _ => {}
            }
        }
        EffectRequest::ResolveMbzhonghui {
            player,
            target,
            action,
            amount,
            mbsizi_def,
            mbkechang_def,
            ..
        } => {
            let Some(p) = seat_of(player, seats) else {
                return;
            };
            let charge = get_mark(ctx.core, p, mbsizi_def.raw()).max(0) as u32;
            match action {
                0 => {
                    if charge > 0 {
                        let spend = 1u32;
                        add_mark_raw(ctx.core, p, mbsizi_def.raw(), -(spend as i32));
                        set_flag(ctx.core, p, "mbsizi_countdown", spend);
                        set_flag(
                            ctx.core,
                            p,
                            "mbsizi_over_hp",
                            u32::from(spend as i32 > ctx.core.player(p).hp),
                        );
                    }
                }
                1 => {
                    if charge < 4 {
                        add_mark_raw(ctx.core, p, mbsizi_def.raw(), (4 - charge) as i32);
                    }
                }
                2 => {
                    let cd = get_flag(ctx.core, p, "mbsizi_countdown");
                    if cd > 0 {
                        set_flag(ctx.core, p, "mbsizi_countdown", cd - 1);
                        ctx.start_draw(p, 2);
                        if get_flag(ctx.core, p, "mbsizi_over_hp") != 0 {
                            if let Some(t) = seat_of(target, seats) {
                                if ctx.core.player(t).alive {
                                    lose_hp_raw(ctx, t, 1);
                                }
                            }
                        }
                    }
                }
                3 => {
                    if get_flag(ctx.core, p, "mbsizi_countdown") > 0 {
                        modify_current_damage(ctx, 1);
                    }
                }
                4 => {
                    let gain = amount.min(4u32.saturating_sub(charge));
                    if gain > 0 {
                        add_mark_raw(ctx.core, p, mbsizi_def.raw(), gain as i32);
                    }
                    if amount > gain {
                        let ov = get_flag(ctx.core, p, "mbsizi_overflow");
                        set_flag(ctx.core, p, "mbsizi_overflow", ov + 1);
                    }
                }
                5 => {
                    set_flag(
                        ctx.core,
                        p,
                        "mbzhonghui_wake_round",
                        ctx.core.turn.round.saturating_add(1),
                    );
                }
                6 => {
                    if get_flag(ctx.core, p, "mbzhonghui_awakened") == 0 {
                        set_flag(ctx.core, p, "mbzhonghui_awakened", 1);
                        set_flag(ctx.core, p, "faction_override", 4);
                        grant_skill_raw(ctx, p, mbkechang_def.raw(), false);
                    }
                }
                7 => {
                    if let Some(t) = seat_of(target, seats) {
                        set_flag(
                            ctx.core,
                            p,
                            &format!("sha_unrespondable:{amount}:{t}"),
                            1,
                        );
                    }
                }
                8 => {
                    let spend = amount.clamp(1, charge.min(4).max(1));
                    add_mark_raw(ctx.core, p, mbsizi_def.raw(), -(spend as i32));
                    set_flag(ctx.core, p, "mbsizi_countdown", spend);
                    set_flag(
                        ctx.core,
                        p,
                        "mbsizi_over_hp",
                        u32::from(spend as i32 > ctx.core.player(p).hp),
                    );
                }
                _ => {}
            }
        }
        EffectRequest::ResolveDddLiuhong {
            player,
            actor,
            action,
            ..
        } => {
            let Some(p) = seat_of(player, seats) else {
                return;
            };
            match action {
                0..=2 => {
                    set_flag(ctx.core, p, "ddd_shixing_mask", u32::from(action) + 1);
                }
                3 => {
                    let q = seat_of(actor, seats)
                        .filter(|&a| ctx.core.player(a).alive && !ctx.core.player(a).linked)
                        .or_else(|| {
                            seats_from(ctx.core, p)
                                .into_iter()
                                .find(|&q| ctx.core.player(q).alive && !ctx.core.player(q).linked)
                        });
                    if let Some(q) = q {
                        ctx.core.player_mut(q).linked = true;
                        ctx.emit(EventData::Linked {
                            player: q,
                            linked: true,
                        });
                    }
                }
                4 => {
                    let qun_count = (0..seats)
                        .filter(|&q| {
                            ctx.core.player(q).alive
                                && ctx.core.effective_faction(ctx.catalog, q)
                                    == Some(Faction::Qun)
                        })
                        .count() as u32;
                    ctx.start_draw(p, qun_count.max(1));
                }
                5 => {
                    if let Some(a) = seat_of(actor, seats) {
                        if let Some(c) = ctx.core.player(a).hand.first().copied() {
                            ctx.discard_from_hand(a, c);
                        }
                    }
                }
                _ => {}
            }
        }
        EffectRequest::ResolveTwguohuai {
            player,
            source,
            card,
            mark_def,
            helper_skill_def,
            action,
            ..
        } => {
            let Some(p) = seat_of(player, seats) else {
                return;
            };
            match action {
                0 => {
                    ctx.start_draw(p, 2);
                    if ctx.core.players.iter().any(|pl| pl.turn_damage > 0) {
                        add_mark_raw(ctx.core, p, mark_def, 1);
                    } else if card.is_some() {
                        set_flag(ctx.core, p, "twguohuai_armed_card", card.raw());
                    }
                }
                1 => {
                    set_flag(ctx.core, p, "twguohuai_armed_card", 0);
                    add_mark_raw(ctx.core, p, mark_def, 1);
                }
                2 => {
                    set_flag(ctx.core, p, "twguohuai_armed_card", 0);
                }
                3 => {
                    add_mark_raw(ctx.core, p, mark_def, 1);
                    grant_skill_raw(ctx, p, helper_skill_def, true);
                    if let Some(s) = seat_of(source, seats) {
                        set_flag(ctx.core, p, "twguohuai_last_source", s as u32 + 1);
                    }
                }
                4 => {
                    add_mark_raw(ctx.core, p, mark_def, -1);
                    set_flag(ctx.core, p, "skip_current_phase", 1);
                }
                _ => {}
            }
        }
        EffectRequest::ResolveDddXuelingyun {
            player,
            actor,
            card,
            cards,
            action,
            ..
        } => {
            let Some(p) = seat_of(player, seats) else {
                return;
            };
            match action {
                0 => {
                    let mut gained = Vec::new();
                    for c in cards {
                        let cid = c.raw();
                        if ctx.core.discard.contains(&cid) && detach(ctx.core, cid) {
                            ctx.core.player_mut(p).hand.push(cid);
                            gained.push(cid);
                        }
                    }
                    if gained.len() == 1 {
                        if let Some(cd) = ctx.catalog.cards.get(gained[0] as usize) {
                            set_flag(ctx.core, p, "ddd_xuelingyun_rank", cd.rank as u32);
                        }
                    }
                }
                1 => set_flag(ctx.core, p, "ddd_xuelingyun_rank", 0),
                2 => {
                    if let Some(cd) = ctx.catalog.cards.get(card.raw() as usize) {
                        let bit = if matches!(cd.suit, Suit::Spade | Suit::Club) {
                            2
                        } else {
                            1
                        };
                        let cur = get_flag(ctx.core, p, "ddd_anzhi_colors");
                        set_flag(ctx.core, p, "ddd_anzhi_colors", cur | bit);
                    }
                }
                3 => {
                    let t = seat_of(actor, seats).unwrap_or(p);
                    set_flag(ctx.core, p, "ddd_anzhi_target", t as u32 + 1);
                }
                4 => {
                    if let Some(a) = seat_of(actor, seats) {
                        remove_from_current_card_targets(ctx, card.raw(), a);
                    }
                    set_flag(ctx.core, p, "ddd_anzhi_target", 0);
                }
                5 => set_flag(ctx.core, p, "ddd_anzhi_target", 0),
                _ => {}
            }
        }
        EffectRequest::ResolvePottaishici {
            player,
            target,
            action,
            cards,
            zhanlie_def,
            ..
        } => {
            let Some(p) = seat_of(player, seats) else {
                return;
            };
            match action {
                0 => {
                    set_flag(ctx.core, p, "pottaishici_quota", 4);
                    set_flag(ctx.core, p, "pottaishici_seen", 0);
                }
                1 => {
                    let mut count = 0i32;
                    for c in cards {
                        if let Some(cd) = ctx.catalog.cards.get(c.raw() as usize) {
                            if cd.kind.is_sha_like() {
                                count += 1;
                            }
                        }
                    }
                    if count > 0 {
                        add_mark_raw(ctx.core, p, zhanlie_def.raw(), count);
                    }
                }
                2 => {
                    let t = seat_of(target, seats).or_else(|| first_other_alive(ctx.core, p));
                    if let Some(t) = t {
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
                3 => {
                    let Some(t) = seat_of(target, seats) else {
                        return;
                    };
                    for d in [p, t] {
                        let draw_n = (4usize.saturating_sub(ctx.core.player(d).hand.len())).min(3);
                        if draw_n > 0 {
                            ctx.start_draw(d, draw_n as u32);
                        }
                    }
                    ctx.emit(EventData::CardUsed(crate::event::CardUseData {
                        card: None,
                        player: p,
                        targets: vec![t],
                        nature: DamageNature::Normal,
                        use_as: Some(CardKind::Duel),
                        cancelled: false,
                    }));
                }
                4 => {
                    if is_wounded(ctx.core, p) {
                        ctx.recover(p, 1, None);
                    } else {
                        set_flag(ctx.core, p, "pottaishici_upgraded", 1);
                    }
                }
                5 => ctx.start_draw(p, 3),
                6 => {
                    if let Some(t) = seat_of(target, seats) {
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
                7 => {
                    if let Some(c) = cards.first() {
                        if ctx.core.player(p).hand.contains(&c.raw()) {
                            ctx.discard_from_hand(p, c.raw());
                        }
                    }
                }
                _ => {}
            }
        }
        other => super::commit_batch15_rest::commit_batch15_rest(ctx, executing, seats, other),
    }
}

pub(super) fn get_flag(core: &GameStateCore, p: PlayerId, key: &str) -> u32 {
    core.vm_aux
        .flags
        .get(&(p, key.to_string()))
        .copied()
        .unwrap_or(0)
}

pub(super) fn set_flag(core: &mut GameStateCore, p: PlayerId, key: &str, val: u32) {
    if val == 0 {
        core.vm_aux.flags.remove(&(p, key.to_string()));
    } else {
        core.vm_aux.flags.insert((p, key.to_string()), val);
    }
}

pub(super) fn get_mark(core: &GameStateCore, p: PlayerId, mark: u32) -> i32 {
    core.vm_aux.marks.get(&(p, mark)).copied().unwrap_or(0)
}

pub(super) fn add_mark_raw(core: &mut GameStateCore, p: PlayerId, mark: u32, delta: i32) {
    if mark == 0 || delta == 0 {
        return;
    }
    let next = get_mark(core, p, mark).saturating_add(delta);
    if next <= 0 {
        core.vm_aux.marks.remove(&(p, mark));
    } else {
        core.vm_aux.marks.insert((p, mark), next);
    }
}

pub(super) fn grant_skill_raw(ctx: &mut Ctx<'_>, p: PlayerId, raw: u32, turn_only: bool) {
    if raw == 0 {
        return;
    }
    if let Some(def) = resolve_skill_def(ctx.catalog, raw) {
        ctx.ensure_skill(
            (p, def),
            SkillInstance {
                owner: p,
                enabled: true,
                once_used: false,
                expires_end_of_turn: turn_only,
                rounds: 0,
            },
        );
    } else {
        ctx.core
            .vm_aux
            .flags
            .insert((p, format!("skill:{raw}")), 1);
        if turn_only {
            ctx.core
                .vm_aux
                .flags
                .insert((p, format!("skill_turn:{raw}")), 1);
        }
    }
}

pub(super) fn remove_skill_raw(ctx: &mut Ctx<'_>, p: PlayerId, raw: u32) {
    if raw == 0 {
        return;
    }
    if let Some(def) = resolve_skill_def(ctx.catalog, raw) {
        ctx.remove_skill(p, def);
    }
    ctx.core.vm_aux.flags.remove(&(p, format!("skill:{raw}")));
    ctx.core
        .vm_aux
        .flags
        .remove(&(p, format!("skill_turn:{raw}")));
}

pub(super) fn lose_hp_raw(ctx: &mut Ctx<'_>, p: PlayerId, amount: u32) {
    ctx.emit(EventData::LoseHp { player: p, amount });
}

pub(super) fn modify_current_damage(ctx: &mut Ctx<'_>, delta: i32) {
    for frame in ctx.res.stack.iter_mut().rev() {
        if let EventData::Damage(d) = &mut frame.data {
            d.amount = (d.amount + delta).max(0);
            break;
        }
    }
}

pub(super) fn deal_normal_damage(
    ctx: &mut Ctx<'_>,
    src: Option<PlayerId>,
    target: PlayerId,
    amount: i32,
) {
    ctx.emit(EventData::Damage(crate::machine::damage_data_with(
        src,
        target,
        amount,
        DamageNature::Normal,
        None,
    )));
}

pub(super) fn gain_same_suit_or_draw(ctx: &mut Ctx<'_>, p: PlayerId, cid: u32) {
    let want_suit = ctx.catalog.cards.get(cid as usize).map(|cd| cd.suit);
    if let Some(suit) = want_suit {
        if let Some(idx) = ctx
            .core
            .deck
            .iter()
            .rposition(|&c| ctx.catalog.cards[c as usize].suit == suit)
        {
            let c = ctx.core.deck.remove(idx);
            ctx.core.player_mut(p).hand.push(c);
            return;
        }
    }
    ctx.start_draw(p, 1);
}

pub(super) fn cancel_current_card_use(ctx: &mut Ctx<'_>, cid: u32) {
    for frame in ctx.res.stack.iter_mut().rev() {
        if let EventData::CardUsed(u) = &mut frame.data {
            if cid == 0 || u.card == Some(cid) {
                u.cancelled = true;
                u.targets.clear();
                break;
            }
        }
    }
}

pub(super) fn remove_from_current_card_targets(ctx: &mut Ctx<'_>, cid: u32, target: PlayerId) {
    for frame in ctx.res.stack.iter_mut().rev() {
        if let EventData::CardUsed(u) = &mut frame.data {
            if cid == 0 || u.card == Some(cid) {
                u.targets.retain(|&q| q != target);
                break;
            }
        }
    }
}

pub(super) fn add_to_current_card_targets(ctx: &mut Ctx<'_>, cid: u32, target: PlayerId) {
    for frame in ctx.res.stack.iter_mut().rev() {
        if let EventData::CardUsed(u) = &mut frame.data {
            if cid == 0 || u.card == Some(cid) {
                if !u.targets.contains(&target) {
                    u.targets.push(target);
                }
                break;
            }
        }
    }
}

pub(super) fn seats_from(core: &GameStateCore, start: PlayerId) -> Vec<PlayerId> {
    let n = core.players.len();
    (0..n).map(|i| ((start as usize + i) % n) as u8).collect()
}

pub(super) fn is_wounded(core: &GameStateCore, p: PlayerId) -> bool {
    let ps = core.player(p);
    ps.hp < ps.max_hp as i32
}

pub(super) fn first_other_alive(core: &GameStateCore, p: PlayerId) -> Option<PlayerId> {
    seats_from(core, p)
        .into_iter()
        .find(|&q| q != p && core.player(q).alive)
}
