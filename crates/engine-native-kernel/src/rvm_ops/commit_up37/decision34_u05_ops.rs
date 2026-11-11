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
        let Some(c) = choose_own_card(ctx, owner, allow_equip) else {
            break;
        };
        if ctx.core.player(owner).hand.contains(&c) {
            ctx.take_hand_card(owner, c);
            disc.push(c);
        } else if allow_equip && ctx.core.player(owner).equips.contains(&c) {
            ctx.take_owned_card(owner, c);
            disc.push(c);
        } else {
            break;
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
        let Some(c) = choose_own_card(ctx, seat, true) else {
            break;
        };
        if ctx.core.player(seat).hand.contains(&c) {
            ctx.take_hand_card(seat, c);
            disc.push(c);
        } else if ctx.core.player(seat).equips.contains(&c) {
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
    let Some(t) = hostile_target(ctx, owner) else {
        return false;
    };
    if steal_to_hand {
        steal_target_he(ctx, owner, t)
    } else {
        discard_target_he(ctx, owner, t).is_some()
    }
}

pub(super) fn u05_store_pile(ctx: &mut Ctx<'_>, owner: PlayerId, pile_name: &str) -> bool {
    let card_opt = if let Some(c) = choose_own_card(ctx, owner, false) {
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
    if let Some(t) = hostile_target(ctx, owner) {
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

