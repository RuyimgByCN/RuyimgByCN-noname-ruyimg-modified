//! Shared card-conservation-safe kernel primitives for `U05 杀海拾遗`
//! (`archive/upstream/noname/apps/core/extension/杀海拾遗`).

use super::commit::discard_some;
use super::*;

pub(super) fn u05_mark_invoked(ctx: &mut Ctx<'_>, owner: PlayerId, skill: &str) {
    set_flag(ctx.core, owner, &format!("u05_skill:{skill}"), 1);
    let cnt = flag(ctx.core, owner, "u05_trigger_count") + 1;
    set_flag(ctx.core, owner, "u05_trigger_count", cnt);
}

pub(super) fn u05_check_usable(ctx: &mut Ctx<'_>, owner: PlayerId, skill: &str, max_uses: i32) -> bool {
    let key = format!("u05_usable:{skill}");
    let used = flag(ctx.core, owner, &key);
    if used >= max_uses.max(0) as u32 {
        set_flag(ctx.core, owner, &format!("u05_blocked_usable:{skill}"), 1);
        return false;
    }
    set_flag(ctx.core, owner, &key, used + 1);
    true
}

pub(super) fn u05_check_limited(ctx: &mut Ctx<'_>, owner: PlayerId, skill: &str) -> bool {
    let key = format!("u05_limited:{skill}");
    if flag(ctx.core, owner, &key) != 0 {
        set_flag(ctx.core, owner, &format!("u05_blocked_limited:{skill}"), 1);
        return false;
    }
    set_flag(ctx.core, owner, &key, 1);
    true
}

pub(super) fn u05_pay_discard(
    ctx: &mut Ctx<'_>,
    owner: PlayerId,
    skill: &str,
    need: usize,
    allow_equip: bool,
) -> bool {
    let avail = ctx.core.player(owner).hand.len()
        + if allow_equip {
            ctx.core.player(owner).equips.len()
        } else {
            0
        };
    if avail < need {
        set_flag(ctx.core, owner, &format!("u05_blocked_cost:{skill}"), 1);
        return false;
    }
    let mut disc = Vec::with_capacity(need);
    for _ in 0..need {
        if let Some(c) = ctx.core.player(owner).hand.first().copied() {
            ctx.take_hand_card(owner, c);
            disc.push(c);
        } else if allow_equip {
            if let Some(c) = ctx.core.player(owner).equips.first().copied() {
                ctx.take_owned_card(owner, c);
                disc.push(c);
            }
        }
    }
    if !disc.is_empty() {
        discard_some(ctx, owner, disc);
    }
    true
}

pub(super) fn u05_discard_up_to(ctx: &mut Ctx<'_>, seat: PlayerId, max_n: usize) -> usize {
    let mut disc = Vec::new();
    while disc.len() < max_n {
        if let Some(c) = ctx.core.player(seat).hand.first().copied() {
            ctx.take_hand_card(seat, c);
            disc.push(c);
        } else if let Some(c) = ctx.core.player(seat).equips.first().copied() {
            ctx.take_owned_card(seat, c);
            disc.push(c);
        } else {
            break;
        }
    }
    let n = disc.len();
    if n > 0 {
        discard_some(ctx, seat, disc);
    }
    n
}

pub(super) fn u05_prey_other(ctx: &mut Ctx<'_>, owner: PlayerId, steal_to_hand: bool) -> bool {
    let Some(t) = first_other_alive(ctx.core, owner) else {
        return false;
    };
    let card_opt = if let Some(c) = ctx.core.player(t).hand.first().copied() {
        ctx.take_hand_card(t, c);
        Some(c)
    } else if let Some(c) = ctx.core.player(t).equips.first().copied() {
        ctx.take_owned_card(t, c);
        Some(c)
    } else {
        None
    };
    if let Some(c) = card_opt {
        if steal_to_hand {
            ctx.core.player_mut(owner).hand.push(c);
        } else {
            discard_some(ctx, t, vec![c]);
        }
        true
    } else {
        false
    }
}

pub(super) fn u05_store_pile(ctx: &mut Ctx<'_>, owner: PlayerId, pile_name: &str) -> bool {
    let card_opt = if let Some(c) = ctx.core.player(owner).hand.first().copied() {
        ctx.take_hand_card(owner, c);
        Some(c)
    } else {
        ctx.core.deck.pop()
    };
    if let Some(c) = card_opt {
        ctx.core
            .vm_aux
            .piles
            .entry((owner, pile_name.to_string()))
            .or_default()
            .push(c);
        true
    } else {
        false
    }
}

pub(super) fn u05_damage_other(
    ctx: &mut Ctx<'_>,
    owner: PlayerId,
    amount: i32,
    nature: DamageNature,
) {
    if let Some(t) = first_other_alive(ctx.core, owner) {
        ctx.emit(EventData::Damage(crate::machine::damage_data_with(
            Some(owner),
            t,
            amount,
            nature,
            None,
        )));
    }
}

pub(super) fn u05_damage_aoe(
    ctx: &mut Ctx<'_>,
    owner: PlayerId,
    amount: i32,
    nature: DamageNature,
) {
    let others: Vec<PlayerId> = (0..ctx.core.players.len() as PlayerId)
        .filter(|&p| p != owner && ctx.core.player(p).alive)
        .collect();
    for t in others {
        ctx.emit(EventData::Damage(crate::machine::damage_data_with(
            Some(owner),
            t,
            amount,
            nature,
            None,
        )));
    }
}

pub(super) fn u05_recover(ctx: &mut Ctx<'_>, seat: PlayerId, amount: i32) {
    if ctx.core.player(seat).hp < ctx.core.player(seat).max_hp as i32 && amount > 0 {
        ctx.recover(seat, amount as u32, None);
    }
}

pub(super) fn u05_lose_hp(ctx: &mut Ctx<'_>, seat: PlayerId, amount: i32) {
    if amount > 0 {
        ctx.emit(EventData::LoseHp {
            player: seat,
            amount: amount as u32,
        });
    }
}

pub(super) fn u05_change_hujia(ctx: &mut Ctx<'_>, seat: PlayerId, delta: i32) {
    let cur = i32::from(ctx.core.player(seat).hujia);
    let next = (cur + delta).clamp(0, 99) as u8;
    ctx.core.player_mut(seat).hujia = next;
}

pub(super) fn u05_change_max_hp(ctx: &mut Ctx<'_>, seat: PlayerId, delta: i32) {
    let cur = ctx.core.player(seat).max_hp as i32;
    let next = (cur + delta).max(1) as u32;
    ctx.core.player_mut(seat).max_hp = next;
    if ctx.core.player(seat).hp > next as i32 {
        ctx.core.player_mut(seat).hp = next as i32;
    }
}

pub(super) fn u05_turn_over(ctx: &mut Ctx<'_>, seat: PlayerId) {
    let fd = !ctx.core.player(seat).face_down;
    ctx.core.player_mut(seat).face_down = fd;
}

pub(super) fn u05_toggle_link(ctx: &mut Ctx<'_>, seat: PlayerId) {
    let lk = !ctx.core.player(seat).linked;
    ctx.core.player_mut(seat).linked = lk;
}

#[derive(Clone, Copy)]
pub(super) enum U05Extra {
    None,
    LoseHpSelf(i32),
    LoseHpOther(i32),
    GainMaxHp(i32),
    LoseMaxHp(i32),
    LoseMaxHpOther(i32),
    StealOther,
    DiscardOther(usize),
    TurnOverSelf,
    TurnOverOther,
    LinkOther,
    StorePile(&'static str),
    ModFlag(&'static str, i32),
    ViewAs(&'static str),
    GrantBuff(&'static str, i32),
    GrantOtherBuff(&'static str, i32),
    JudgeEffect(&'static str),
    PindianOther,
    ExtraTurn,
}

#[allow(clippy::too_many_arguments)]
pub(super) fn u05_exec(
    ctx: &mut Ctx<'_>,
    owner: PlayerId,
    skill: &str,
    usable: u8,
    limited: bool,
    cost_cards: u8,
    draw_self: u8,
    draw_other: u8,
    dmg: i8,
    nature: DamageNature,
    recover_self: u8,
    hujia_delta: i8,
    extra: U05Extra,
) {
    if !guard_phase_auto_trigger(
        ctx,
        owner,
        "u05",
        skill,
        matches!(extra, U05Extra::LoseHpSelf(_)),
    ) {
        return;
    }
    u05_mark_invoked(ctx, owner, skill);
    if limited && !u05_check_limited(ctx, owner, skill) {
        return;
    }
    if usable > 0 && !u05_check_usable(ctx, owner, skill, i32::from(usable)) {
        return;
    }
    if cost_cards > 0 && !u05_pay_discard(ctx, owner, skill, usize::from(cost_cards), true) {
        return;
    }
    if hujia_delta != 0 {
        u05_change_hujia(ctx, owner, i32::from(hujia_delta));
    }
    if recover_self > 0 {
        u05_recover(ctx, owner, i32::from(recover_self));
    }
    if dmg > 0 {
        u05_damage_other(ctx, owner, i32::from(dmg), nature);
    } else if dmg < 0 {
        u05_damage_aoe(ctx, owner, i32::from(-dmg), nature);
    }
    if draw_self > 0 {
        ctx.start_draw(owner, u32::from(draw_self));
    }
    if draw_other > 0 {
        if let Some(t) = first_other_alive(ctx.core, owner) {
            ctx.start_draw(t, u32::from(draw_other));
        }
    }
    match extra {
        U05Extra::None => {}
        U05Extra::LoseHpSelf(n) => u05_lose_hp(ctx, owner, n),
        U05Extra::LoseHpOther(n) => {
            if let Some(t) = first_other_alive(ctx.core, owner) {
                u05_lose_hp(ctx, t, n);
            }
        }
        U05Extra::GainMaxHp(n) => u05_change_max_hp(ctx, owner, n),
        U05Extra::LoseMaxHp(n) => u05_change_max_hp(ctx, owner, -n),
        U05Extra::LoseMaxHpOther(n) => {
            if let Some(t) = first_other_alive(ctx.core, owner) {
                u05_change_max_hp(ctx, t, -n);
            }
        }
        U05Extra::StealOther => {
            let _ = u05_prey_other(ctx, owner, true);
        }
        U05Extra::DiscardOther(n) => {
            if let Some(t) = first_other_alive(ctx.core, owner) {
                let _ = u05_discard_up_to(ctx, t, n);
            }
        }
        U05Extra::TurnOverSelf => u05_turn_over(ctx, owner),
        U05Extra::TurnOverOther => {
            if let Some(t) = first_other_alive(ctx.core, owner) {
                u05_turn_over(ctx, t);
            }
        }
        U05Extra::LinkOther => {
            if let Some(t) = first_other_alive(ctx.core, owner) {
                u05_toggle_link(ctx, t);
            }
        }
        U05Extra::StorePile(pile_key) => {
            let _ = u05_store_pile(ctx, owner, pile_key);
        }
        U05Extra::ModFlag(flag_key, val) => {
            set_flag(ctx.core, owner, flag_key, val.max(0) as u32);
        }
        U05Extra::ViewAs(card_name) => {
            set_flag(ctx.core, owner, &format!("u05_viewas:{card_name}"), 1);
        }
        U05Extra::GrantBuff(buff_key, turns) => {
            set_flag(ctx.core, owner, buff_key, turns.max(0) as u32);
        }
        U05Extra::GrantOtherBuff(buff_key, turns) => {
            if let Some(t) = first_other_alive(ctx.core, owner) {
                set_flag(ctx.core, t, buff_key, turns.max(0) as u32);
            }
        }
        U05Extra::JudgeEffect(judge_key) => {
            let mut rank = 7u32;
            if let Some(jcard) = pop_deck(ctx) {
                if let Some(def) = ctx.catalog.cards.get(jcard as usize) {
                    rank = u32::from(def.rank);
                }
                discard_some(ctx, owner, vec![jcard]);
            }
            set_flag(ctx.core, owner, judge_key, rank);
            if rank > 6 {
                ctx.start_draw(owner, 1);
            }
        }
        U05Extra::PindianOther => {
            if let Some(t) = first_other_alive(ctx.core, owner) {
                let my_c = ctx.core.player(owner).hand.first().copied();
                let th_c = ctx.core.player(t).hand.first().copied();
                if let (Some(mc), Some(tc)) = (my_c, th_c) {
                    let my_r = ctx.catalog.cards.get(mc as usize).map(|c| c.rank).unwrap_or(0);
                    let th_r = ctx.catalog.cards.get(tc as usize).map(|c| c.rank).unwrap_or(0);
                    ctx.take_hand_card(owner, mc);
                    discard_some(ctx, owner, vec![mc]);
                    ctx.take_hand_card(t, tc);
                    discard_some(ctx, t, vec![tc]);
                    let won = my_r > th_r;
                    set_flag(ctx.core, owner, &format!("u05_pindian_won:{skill}"), u32::from(won));
                    if won {
                        ctx.start_draw(owner, 1);
                    }
                }
            }
        }
        U05Extra::ExtraTurn => {
            set_flag(ctx.core, owner, "u05_mod:extra_turn", 1);
            ctx.start_draw(owner, 2);
        }
    }
}

