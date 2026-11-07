//! PA12 批⑮（下半部分）：tags 164..=178 的权威落账（由 `commit_batch15` 转发）。

use super::commit::{detach, discard_some, pop_deck, push_pile};
use super::commit_batch15::{
    add_mark_raw, add_to_current_card_targets, cancel_current_card_use, deal_normal_damage,
    first_other_alive, get_flag, get_mark, grant_skill_raw, is_wounded, lose_hp_raw,
    modify_current_damage, remove_from_current_card_targets, remove_skill_raw, set_flag,
    DDD_BAILEI_DEF, DDD_FENGZHENG_GLOBAL_DEF,
};
use super::*;

pub(super) fn commit_batch15_rest(
    ctx: &mut Ctx<'_>,
    executing: &str,
    seats: u8,
    e: EffectRequest,
) {
    match e {
        EffectRequest::ResolveDddXujing {
            player,
            source,
            action,
            ..
        } => {
            let Some(p) = seat_of(player, seats) else {
                return;
            };
            match action {
                0 => {
                    set_flag(ctx.core, p, "ddd_xuyu_pending", 1);
                    ctx.start_draw(p, 1);
                }
                1 => {
                    if get_flag(ctx.core, p, "ddd_xuyu_pending") != 0 {
                        set_flag(ctx.core, p, "ddd_xuyu_pending", 0);
                        if let Some(c) = ctx.core.player(p).hand.first().copied() {
                            ctx.discard_from_hand(p, c);
                        }
                    }
                }
                2 => set_flag(ctx.core, p, "ddd_xuyu_pending", 0),
                3 => set_flag(ctx.core, p, "ddd_xuyu_pending", 1),
                10 => {
                    if let Some(s) = seat_of(source, seats) {
                        if s != p && ctx.core.player(s).alive {
                            if let Some(c) = ctx.core.player(p).hand.first().copied() {
                                ctx.give_hand_card(p, s, c);
                                set_flag(ctx.core, p, "ddd_shijian_watched", s as u32 + 1);
                            }
                        }
                    }
                }
                11 => {
                    set_flag(ctx.core, p, "ddd_shijian_watched", 0);
                    ctx.start_draw(p, 1);
                }
                12 => set_flag(ctx.core, p, "ddd_shijian_watched", 0),
                13 => {
                    if let Some(s) = seat_of(source, seats) {
                        set_flag(ctx.core, p, "ddd_shijian_watched", s as u32 + 1);
                    }
                }
                _ => {}
            }
        }
        EffectRequest::ResolveDddLiuba {
            player,
            actor,
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
                    let mut dropped = Vec::new();
                    for c in cards {
                        if ctx.core.player(p).hand.contains(&c.raw()) {
                            ctx.take_hand_card(p, c.raw());
                            dropped.push(c.raw());
                        }
                    }
                    discard_some(ctx, p, dropped);
                    ctx.start_draw(p, 2);
                }
                1 => ctx.start_draw(p, 2),
                2 => {
                    for q in 0..seats {
                        if ctx.core.player(q).alive {
                            grant_skill_raw(ctx, q, DDD_FENGZHENG_GLOBAL_DEF, false);
                        }
                    }
                }
                3 => {
                    ctx.start_draw(p, 1);
                    if let Some(c) = ctx.core.player(p).hand.first().copied() {
                        ctx.take_hand_card(p, c);
                        push_pile(ctx, p, executing, c);
                    }
                }
                4 => {
                    let a = seat_of(actor, seats).unwrap_or(p);
                    ctx.start_draw(a, 1);
                }
                10 => {
                    for c in cards {
                        if detach(ctx.core, c.raw()) {
                            ctx.core.player_mut(p).hand.push(c.raw());
                        }
                    }
                }
                11..=13 => {
                    if let Some(c) = ctx.core.player(p).hand.first().copied() {
                        ctx.take_hand_card(p, c);
                        push_pile(ctx, p, executing, c);
                    }
                }
                14 => {
                    for (c, dst_id) in cards.iter().zip(targets.iter()) {
                        if let Some(dst) = seat_of(*dst_id, seats) {
                            if detach(ctx.core, c.raw()) {
                                ctx.core.player_mut(dst).hand.push(c.raw());
                            }
                        }
                    }
                }
                15 => {
                    if let Some(c) = cards.first() {
                        let cid = c.raw();
                        if ctx.core.player(p).hand.contains(&cid) {
                            ctx.take_hand_card(p, cid);
                            let key = (p, executing.to_string());
                            let old = ctx
                                .core
                                .vm_aux
                                .piles
                                .get_mut(&key)
                                .and_then(|v| (!v.is_empty()).then(|| v.remove(0)));
                            push_pile(ctx, p, executing, cid);
                            if let Some(old_c) = old {
                                ctx.core.player_mut(p).hand.push(old_c);
                            }
                        }
                    }
                }
                _ => {}
            }
        }
        EffectRequest::ResolveDddLiuye {
            player,
            actor,
            action,
            ..
        } => {
            let Some(p) = seat_of(player, seats) else {
                return;
            };
            match action {
                0 => {
                    if let Some(a) = seat_of(actor, seats) {
                        if a != p {
                            if let Some(c) = ctx.core.player(p).hand.first().copied() {
                                ctx.discard_from_hand(p, c);
                                set_flag(ctx.core, p, "ddd_chashi_target", a as u32 + 1);
                            }
                        }
                    }
                }
                1 => {
                    let a = seat_of(actor, seats).unwrap_or(p);
                    if get_flag(ctx.core, p, "ddd_chashi_target") == a as u32 + 1 {
                        set_flag(ctx.core, p, "ddd_chashi_target", 0);
                        ctx.start_draw(a, 1);
                        ctx.start_draw(p, 1);
                    }
                }
                2 => {
                    if get_flag(ctx.core, p, "ddd_qice_used") == 0 {
                        set_flag(ctx.core, p, "ddd_qice_used", 1);
                        ctx.start_draw(p, 2);
                    }
                }
                3 => set_flag(ctx.core, p, "ddd_qice_used", 0),
                4 => set_flag(ctx.core, p, "ddd_chashi_target", 0),
                _ => {}
            }
        }
        EffectRequest::ResolveDddZhangkai {
            player,
            actor,
            card,
            cards,
            amount,
            action,
            ..
        } => {
            let Some(p) = seat_of(player, seats) else {
                return;
            };
            match action {
                0 => {
                    set_flag(ctx.core, p, "ddd_jiexing_stolen", 1);
                    ctx.recover(p, amount.max(1), None);
                }
                1 => {
                    cancel_current_card_use(ctx, card.raw());
                    for c in cards {
                        let cid = c.raw();
                        if detach(ctx.core, cid) {
                            ctx.core.player_mut(p).equips.push(cid);
                            crate::kernel::grant_equip_skills(ctx, p, cid);
                        }
                    }
                }
                2 | 3 => set_flag(ctx.core, p, "ddd_jiexing_stolen", 0),
                4 => {
                    let t = seat_of(actor, seats)
                        .filter(|&a| a != p && ctx.core.player(a).alive)
                        .or_else(|| first_other_alive(ctx.core, p));
                    if let Some(t) = t {
                        deal_normal_damage(ctx, Some(p), t, 1);
                    }
                }
                5 => {
                    if seat_of(actor, seats).is_none_or(|a| !ctx.core.player(a).alive) {
                        remove_skill_raw(ctx, p, DDD_BAILEI_DEF);
                    }
                }
                _ => {}
            }
        }
        EffectRequest::ResolveHiddenSkillPile {
            player,
            pile_skill_def,
            action,
            card,
            hand_cards,
            pile_cards,
            event_cards,
            ..
        } => {
            let Some(p) = seat_of(player, seats) else {
                return;
            };
            let key_name = pile_name(ctx.catalog, executing, pile_skill_def.raw());
            match action {
                0 => {
                    if ctx
                        .core
                        .vm_aux
                        .piles
                        .get(&(p, key_name.clone()))
                        .is_none_or(|v| v.is_empty())
                    {
                        for _ in 0..2 {
                            if let Some(c) = pop_deck(ctx) {
                                push_pile(ctx, p, &key_name, c);
                            }
                        }
                    }
                }
                1 | 3 => {
                    let sel = pile_cards.first().map(|c| c.raw()).or_else(|| {
                        ctx.core
                            .vm_aux
                            .piles
                            .get(&(p, key_name.clone()))
                            .and_then(|v| v.first().copied())
                    });
                    if let Some(cid) = sel {
                        set_flag(
                            ctx.core,
                            p,
                            &format!("hidden_pile_rev:{}:{cid}", pile_skill_def.raw()),
                            1,
                        );
                        if action == 1 {
                            remove_from_current_card_targets(ctx, card.raw(), p);
                        } else {
                            ctx.start_draw(p, 1);
                        }
                    }
                }
                2 => {
                    let hc = hand_cards
                        .first()
                        .map(|c| c.raw())
                        .or_else(|| ctx.core.player(p).hand.first().copied());
                    let pc = pile_cards.first().map(|c| c.raw()).or_else(|| {
                        ctx.core
                            .vm_aux
                            .piles
                            .get(&(p, key_name.clone()))
                            .and_then(|v| v.first().copied())
                    });
                    if let (Some(h), Some(pl)) = (hc, pc) {
                        if ctx.core.player(p).hand.contains(&h) && detach(ctx.core, pl) {
                            ctx.take_hand_card(p, h);
                            push_pile(ctx, p, &key_name, h);
                            ctx.core.player_mut(p).hand.push(pl);
                        }
                    }
                }
                4 => {
                    let n = event_cards.len().max(1) as u32;
                    ctx.start_draw(p, n);
                }
                _ => {}
            }
        }
        EffectRequest::ResolveTwfuman {
            player,
            target,
            cards,
            action,
            ..
        } => {
            let Some(p) = seat_of(player, seats) else {
                return;
            };
            match action {
                0 => {
                    if let (Some(t), Some(c)) = (seat_of(target, seats), cards.first()) {
                        let cid = c.raw();
                        if p != t && ctx.core.player(p).hand.contains(&cid) {
                            ctx.give_hand_card(p, t, cid);
                            set_flag(ctx.core, p, &format!("twfuman_card:{cid}"), t as u32 + 1);
                        }
                    }
                }
                1 | 2 => {
                    for c in cards {
                        let key = format!("twfuman_card:{}", c.raw());
                        if get_flag(ctx.core, p, &key) != 0 {
                            set_flag(ctx.core, p, &key, 0);
                            if action == 1 {
                                ctx.start_draw(p, 1);
                            }
                        }
                    }
                }
                _ => {}
            }
        }
        EffectRequest::ResolveTwkujian {
            player,
            target,
            cards,
            action,
            ..
        } => {
            let (Some(p), Some(t)) = (seat_of(player, seats), seat_of(target, seats)) else {
                return;
            };
            match action {
                0 => {
                    for c in cards.iter().take(3) {
                        let cid = c.raw();
                        if ctx.core.player(p).hand.contains(&cid) {
                            ctx.give_hand_card(p, t, cid);
                            set_flag(ctx.core, p, &format!("twkujian_card:{cid}"), 1);
                        }
                    }
                }
                1 => {
                    let n = cards.len().max(1) as u32;
                    for c in cards {
                        set_flag(ctx.core, p, &format!("twkujian_card:{}", c.raw()), 0);
                    }
                    ctx.start_draw(p, n);
                    ctx.start_draw(t, n);
                }
                2 => {
                    for c in cards {
                        set_flag(ctx.core, p, &format!("twkujian_card:{}", c.raw()), 0);
                    }
                    for q in [p, t] {
                        if let Some(c) = ctx.core.player(q).hand.first().copied() {
                            ctx.discard_from_hand(q, c);
                        }
                    }
                }
                _ => {}
            }
        }
        EffectRequest::ResolveTwxiawei {
            player,
            wangxing_skill_def,
            action,
            stage,
            ..
        } => {
            let Some(p) = seat_of(player, seats) else {
                return;
            };
            match action {
                0 => {
                    let basics: Vec<CardId> = ctx
                        .core
                        .deck
                        .iter()
                        .rev()
                        .chain(ctx.core.discard.iter().rev())
                        .copied()
                        .filter(|&c| ctx.catalog.cards[c as usize].kind.is_basic_kind())
                        .take(2)
                        .collect();
                    for c in basics {
                        if detach(ctx.core, c) {
                            push_pile(ctx, p, executing, c);
                        }
                    }
                }
                1 => {
                    let key = (p, executing.to_string());
                    let dropped = ctx.core.vm_aux.piles.remove(&key).unwrap_or_default();
                    discard_some(ctx, p, dropped);
                }
                2 => {
                    let st = if stage == 0 { 1 } else { stage };
                    for _ in 0..=st {
                        if let Some(c) = pop_deck(ctx) {
                            push_pile(ctx, p, executing, c);
                        }
                    }
                    grant_skill_raw(ctx, p, wangxing_skill_def, false);
                    add_mark_raw(ctx.core, p, wangxing_skill_def, st as i32);
                }
                3 => modify_current_damage(ctx, 1),
                4 => {
                    let m = get_mark(ctx.core, p, wangxing_skill_def);
                    if m > 0 {
                        add_mark_raw(ctx.core, p, wangxing_skill_def, -m);
                        lose_hp_raw(ctx, p, 1);
                    }
                }
                _ => {}
            }
        }
        EffectRequest::ResolveCaoyi {
            player,
            card,
            helper_skill_def,
            action,
            event_targets,
            ..
        } => {
            let Some(p) = seat_of(player, seats) else {
                return;
            };
            match action {
                0 => {
                    grant_skill_raw(ctx, p, helper_skill_def.raw(), false);
                    set_flag(ctx.core, p, "caoyi_miyi_branch", 1);
                    if is_wounded(ctx.core, p) {
                        ctx.recover(p, 1, None);
                    }
                }
                1 => {
                    let branch = card.raw().min(1);
                    for tid in event_targets {
                        if let Some(t) = seat_of(tid, seats) {
                            if ctx.core.player(t).alive {
                                grant_skill_raw(ctx, t, helper_skill_def.raw(), false);
                                set_flag(ctx.core, t, "caoyi_miyi_branch", branch + 1);
                                if branch == 0 {
                                    ctx.recover(t, 1, None);
                                } else {
                                    deal_normal_damage(ctx, Some(p), t, 1);
                                }
                            }
                        }
                    }
                }
                10 => {
                    let b = get_flag(ctx.core, p, "caoyi_miyi_branch");
                    set_flag(ctx.core, p, "caoyi_miyi_branch", 0);
                    remove_skill_raw(ctx, p, helper_skill_def.raw());
                    if b == 1 {
                        deal_normal_damage(ctx, None, p, 1);
                    } else if b == 2 {
                        ctx.recover(p, 1, None);
                    }
                }
                11 => remove_skill_raw(ctx, p, helper_skill_def.raw()),
                20 => {
                    if let Some(tid) = event_targets.first() {
                        if let Some(t) = seat_of(*tid, seats) {
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
                }
                21 => {
                    let c = get_flag(ctx.core, p, "caoyi_yinjun_count") + 1;
                    set_flag(ctx.core, p, "caoyi_yinjun_count", c);
                }
                _ => {}
            }
        }
        EffectRequest::ResolveDcSbLusu {
            player,
            source,
            action,
            cards,
            amount,
            ..
        } => {
            let Some(p) = seat_of(player, seats) else {
                return;
            };
            match action {
                0 => {
                    ctx.start_draw(p, 2);
                    let t = seat_of(source, seats)
                        .filter(|&s| s != p && ctx.core.player(s).alive)
                        .or_else(|| first_other_alive(ctx.core, p));
                    if let (Some(t), Some(c)) = (t, ctx.core.player(p).hand.first().copied()) {
                        ctx.give_hand_card(p, t, c);
                    }
                }
                1 => {
                    let t = seat_of(source, seats)
                        .filter(|&s| s != p && ctx.core.player(s).alive)
                        .or_else(|| first_other_alive(ctx.core, p));
                    let cid = cards
                        .first()
                        .map(|c| c.raw())
                        .filter(|c| ctx.core.player(p).hand.contains(c))
                        .or_else(|| ctx.core.player(p).hand.first().copied());
                    if let (Some(t), Some(c)) = (t, cid) {
                        ctx.give_hand_card(p, t, c);
                    }
                }
                2 => {
                    let t = seat_of(source, seats)
                        .filter(|&s| s != p && ctx.core.player(s).alive)
                        .or_else(|| first_other_alive(ctx.core, p));
                    let cid = cards
                        .first()
                        .map(|c| c.raw())
                        .filter(|c| ctx.core.player(p).hand.contains(c))
                        .or_else(|| ctx.core.player(p).hand.first().copied());
                    if let (Some(t), Some(cid)) = (t, cid) {
                        ctx.give_hand_card(p, t, cid);
                    }
                }
                10 => {
                    let b = get_flag(ctx.core, p, "dcsblusu_branch");
                    set_flag(ctx.core, p, "dcsblusu_branch", 1 - b.min(1));
                }
                11 | 21 | 22 | 23 => {
                    let b = get_flag(ctx.core, p, "dcsblusu_branch");
                    set_flag(ctx.core, p, "dcsblusu_branch", 1 - b.min(1));
                    ctx.start_draw(p, 1);
                }
                20 => {
                    if let Some(s) = seat_of(source, seats) {
                        if amount > 0 && ctx.core.player(s).alive {
                            ctx.recover(s, amount, None);
                        }
                    }
                }
                _ => {}
            }
        }
        EffectRequest::ResolveDcSbZhouyu {
            player,
            card,
            action,
            event_targets,
            ..
        } => {
            let Some(p) = seat_of(player, seats) else {
                return;
            };
            match action {
                0 => set_flag(ctx.core, p, "dcsbzhouyu_armed", 1),
                1 => {
                    if card.is_some() {
                        *ctx.core
                            .vm_aux
                            .flags
                            .entry((p, format!("card_damage_bonus:{}", card.raw())))
                            .or_insert(0) += 1;
                    }
                    set_flag(ctx.core, p, "dcsbzhouyu_armed", 0);
                }
                10 => {
                    let b = get_flag(ctx.core, p, "dcsbzhouyu_branch");
                    set_flag(ctx.core, p, "dcsbzhouyu_branch", 1 - b.min(1));
                }
                11..=15 | 20..=23 => {
                    let b = get_flag(ctx.core, p, "dcsbzhouyu_branch");
                    set_flag(ctx.core, p, "dcsbzhouyu_branch", 1 - b.min(1));
                    let t = event_targets
                        .first()
                        .and_then(|tid| seat_of(*tid, seats))
                        .or_else(|| first_other_alive(ctx.core, p));
                    if let Some(t) = t {
                        let diff = ctx
                            .core
                            .player(t)
                            .hand
                            .len()
                            .saturating_sub(ctx.core.player(p).hand.len())
                            .clamp(1, 5) as u32;
                        ctx.start_draw(p, diff);
                    }
                }
                _ => {}
            }
        }
        EffectRequest::ResolveMbxiongtu {
            player,
            target,
            stage,
            card,
            cards,
            ..
        } => {
            let Some(p) = seat_of(player, seats) else {
                return;
            };
            match stage {
                0..=2 => {
                    for c in cards {
                        if ctx.core.player(p).hand.contains(&c.raw()) {
                            ctx.discard_from_hand(p, c.raw());
                        }
                    }
                    if let Some(t) = seat_of(target, seats) {
                        if p != t && ctx.core.player(t).alive {
                            deal_normal_damage(ctx, Some(p), t, 1);
                        }
                    }
                }
                3 => {
                    if let Some(t) = seat_of(target, seats) {
                        if card.is_some() && ctx.core.player(t).hand.contains(&card.raw()) {
                            ctx.discard_from_hand(t, card.raw());
                        }
                    }
                    let f = get_flag(ctx.core, p, "mbxiongtu_flags");
                    set_flag(ctx.core, p, "mbxiongtu_flags", f | 1);
                }
                4 => {
                    let f = get_flag(ctx.core, p, "mbxiongtu_flags");
                    set_flag(ctx.core, p, "mbxiongtu_flags", (f & !4) | 1);
                }
                5 => {
                    set_flag(ctx.core, p, "mbxiongtu_flags", 2);
                    ctx.start_draw(p, 1);
                }
                6 | 7 => set_flag(ctx.core, p, "mbxiongtu_flags", 0),
                _ => {}
            }
        }
        EffectRequest::ResolvePothongyi {
            player,
            stage,
            skill,
        } => {
            let Some(p) = seat_of(player, seats) else {
                return;
            };
            let mark = ctx
                .catalog
                .skill_by_name(executing)
                .map(|d| d as u32)
                .unwrap_or_else(|| skill.raw().max(1));
            let cur = get_mark(ctx.core, p, mark).clamp(0, 5) as u32;
            match stage {
                0 => {
                    if cur < 4 {
                        add_mark_raw(ctx.core, p, mark, (4 - cur) as i32);
                    }
                }
                1 => {
                    if cur < 5 {
                        add_mark_raw(ctx.core, p, mark, 1);
                    }
                }
                2 => {
                    if cur > 0 {
                        set_flag(ctx.core, p, "pothongyi_branch", 1);
                        set_flag(ctx.core, p, "pothongyi_count", cur);
                        ctx.start_draw(p, cur);
                    }
                }
                3 => {
                    let b = get_flag(ctx.core, p, "pothongyi_branch");
                    let n = get_flag(ctx.core, p, "pothongyi_count").max(1);
                    set_flag(ctx.core, p, "pothongyi_branch", 0);
                    if b == 1 {
                        let m = get_mark(ctx.core, p, mark);
                        if m > 0 {
                            add_mark_raw(ctx.core, p, mark, -m);
                        }
                    } else if b == 2 {
                        ctx.start_draw(p, n);
                    }
                }
                _ => {}
            }
        }
        EffectRequest::ResolveSpyingjia {
            player,
            action,
            helper_def,
            card,
            target,
            skill,
            ..
        } => {
            let Some(p) = seat_of(player, seats) else {
                return;
            };
            match action {
                0 => set_flag(ctx.core, p, "spyingjia_repeated", 0),
                1 => set_flag(ctx.core, p, "spyingjia_repeated", 1),
                2 => {
                    let cid = card
                        .is_some()
                        .then(|| card.raw())
                        .filter(|c| ctx.core.player(p).hand.contains(c))
                        .or_else(|| ctx.core.player(p).hand.first().copied());
                    if let Some(c) = cid {
                        ctx.discard_from_hand(p, c);
                        grant_skill_raw(ctx, p, helper_def.raw(), false);
                        set_flag(ctx.core, p, "spyingjia_extra_turn", 1);
                    }
                }
                3 => {
                    ctx.start_draw(p, 2);
                    remove_skill_raw(ctx, p, skill.raw());
                }
                4 => {
                    let t = seat_of(target, seats).unwrap_or(p);
                    if card.is_some() && ctx.core.player(p).hand.contains(&card.raw()) {
                        ctx.discard_from_hand(p, card.raw());
                    }
                    grant_skill_raw(ctx, t, helper_def.raw(), false);
                    set_flag(ctx.core, t, "spyingjia_extra_turn", 1);
                }
                _ => {}
            }
        }
        EffectRequest::ResolveDddCaoshuang {
            player,
            actor,
            card,
            action,
            ..
        } => {
            let Some(p) = seat_of(player, seats) else {
                return;
            };
            match action {
                0 => {
                    if let Some(a) = seat_of(actor, seats) {
                        if a != p && ctx.core.player(a).alive {
                            let cid = card
                                .is_some()
                                .then(|| card.raw())
                                .filter(|c| ctx.core.player(p).hand.contains(c))
                                .or_else(|| ctx.core.player(p).hand.first().copied());
                            if let Some(c) = cid {
                                ctx.give_hand_card(p, a, c);
                                set_flag(ctx.core, p, "ddd_zhuanshe_actor", a as u32 + 1);
                            }
                        }
                    }
                }
                1 => {
                    let q = seat_of(actor, seats)
                        .filter(|&a| a != p && ctx.core.player(a).alive)
                        .or_else(|| first_other_alive(ctx.core, p));
                    if let Some(q) = q {
                        add_to_current_card_targets(ctx, card.raw(), q);
                    }
                }
                2 => {
                    if let Some(a) = seat_of(actor, seats) {
                        set_flag(ctx.core, p, "ddd_zhuanshe_actor", 0);
                        if ctx.core.player(a).alive {
                            deal_normal_damage(ctx, Some(p), a, 1);
                        }
                    }
                }
                3 => {
                    if ctx.core.player(p).hand.is_empty() {
                        ctx.start_draw(p, 1);
                    }
                }
                _ => {}
            }
        }
        other => super::commit_batch16::commit_batch16_effect(ctx, executing, seats, other),
    }
}
