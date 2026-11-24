//! Shared helpers for `L11` (`太虚幻境`) cross-package NPC skills Part B (`l11_npc_b`).

use super::super::l11_common::{
    alive_all, alive_others, discard_player_cards, emit_damage, emit_lose_hp, ensure_perm_skill,
    gain_card_from_deck_or_discard, has_equip_named, has_named_skill, he_cards, hej_cards,
    is_locked_unblocked_l11_owner, is_unblocked_l11_owner, move_cards_to_hand, remove_named_skill,
};
use super::super::l11_common_b2::is_ordinary_trick_kind;
use super::super::*;
use super::build_ask_l11_npc_b;
use crate::catalog::{CardKind, Catalog, EquipSlot, Suit};
use crate::event::{CardUseData, DamageNature, EventData};
use crate::ids::{CardId, EventId, PlayerId};
use crate::machine::{damage_data_with, Ctx, Flow, LogEvent};
use crate::state::GameStateCore;

pub(super) fn card_use_virtual(
    player: PlayerId,
    targets: Vec<PlayerId>,
    nature: DamageNature,
    kind: CardKind,
) -> CardUseData {
    CardUseData {
        card: None,
        player,
        targets,
        nature,
        use_as: Some(kind),
        cancelled: false,
    }
}

pub(super) fn is_huogong_card(catalog: &Catalog, card: CardId) -> bool {
    catalog.cards.get(card as usize).is_some_and(|cd| cd.name == "huogong")
}

pub(super) const SHEFU_NAMES: [CardKind; 14] = [
    CardKind::Sha,
    CardKind::Shan,
    CardKind::Tao,
    CardKind::Jiu,
    CardKind::Wuzhong,
    CardKind::Dismantle,
    CardKind::Snatch,
    CardKind::Duel,
    CardKind::Nullify,
    CardKind::SavageAttack,
    CardKind::ArcheryAttack,
    CardKind::Taoyuan,
    CardKind::Wugu,
    CardKind::Chain,
];

#[inline]
pub(super) fn flag(core: &GameStateCore, p: PlayerId, key: &str) -> u32 {
    core.vm_aux
        .flags
        .get(&(p, key.to_string()))
        .copied()
        .unwrap_or(0)
}

#[inline]
pub(super) fn set_flag(core: &mut GameStateCore, p: PlayerId, key: &str, val: u32) {
    if val == 0 {
        core.vm_aux.flags.remove(&(p, key.to_string()));
    } else {
        core.vm_aux.flags.insert((p, key.to_string()), val);
    }
}

#[inline]
pub(super) fn turn_stamp(core: &GameStateCore) -> u32 {
    core.turn
        .round
        .saturating_mul(64)
        .saturating_add(u32::from(core.turn.current) + 1)
}

#[inline]
pub(super) fn phase_stamp(core: &GameStateCore) -> u32 {
    turn_stamp(core)
        .saturating_mul(8)
        .saturating_add((core.turn.phase as u32) + 1)
}

#[inline]
pub(super) fn suit_bit(suit: Suit) -> u32 {
    match suit {
        Suit::Spade => 1,
        Suit::Heart => 2,
        Suit::Club => 4,
        Suit::Diamond => 8,
    }
}

#[inline]
pub(super) fn is_red_suit(suit: Suit) -> bool {
    matches!(suit, Suit::Heart | Suit::Diamond)
}

/// Group per `get.type(c, 'trick')`: 0 = basic, 1 = trick/delay, 2 = equip.
#[inline]
pub(super) fn card_type_group(kind: CardKind) -> u8 {
    if kind.is_basic_kind() {
        0
    } else if kind.equip_slot().is_some() {
        2
    } else {
        1
    }
}

pub(super) fn wait_ask(
    m: &mut HostMachine,
    ctx: &mut Ctx<'_>,
    stage: HostStage,
    ask_id: u32,
) -> Flow {
    set_flag(ctx.core, m.owner, "l11b_ask_id", ask_id);
    m.stage = stage;
    if let Some((observer, spec)) = build_ask_l11_npc_b(
        stage,
        m.owner,
        m.target,
        &m.targets,
        m.card,
        &m.cards,
        ctx.core,
        ctx.catalog,
    ) {
        Flow::WaitChoice { observer, spec }
    } else {
        set_flag(ctx.core, m.owner, "l11b_ask_id", 0);
        Flow::Done
    }
}

pub(super) fn clear_ask(ctx: &mut Ctx<'_>, owner: PlayerId) {
    set_flag(ctx.core, owner, "l11b_ask_id", 0);
}

pub(super) fn spawn_npc_b_park(
    ctx: &mut Ctx<'_>,
    def_key: &str,
    owner: PlayerId,
    target: Option<PlayerId>,
    card: Option<CardId>,
    cards: Vec<CardId>,
    targets: Vec<PlayerId>,
) {
    ctx.res.parks.push(crate::resolution::Park {
        resume_depth: ctx.res.stack.len(),
        slot: crate::resolution::Slot::Intrinsic {
            frame: crate::resolution::NO_FRAME,
        },
        awaiting_answer: false,
        machine: Box::new(HostMachine {
            def_key: def_key.to_string(),
            behavior: HostBehavior::L11CrossNpcB,
            owner,
            target,
            card,
            cards,
            targets,
            stage: HostStage::Act,
        }),
    });
}

pub(super) fn nullify_current_card_on_owner(ctx: &mut Ctx<'_>, owner: PlayerId) {
    for f in ctx.res.stack.iter_mut().rev() {
        if let EventData::CardUsed(ref mut u) = f.data {
            u.targets.retain(|&t| t != owner);
            if u.targets.is_empty() {
                u.cancelled = true;
            }
            break;
        }
    }
}

pub(super) fn cancel_all_current_card_targets(ctx: &mut Ctx<'_>) {
    for f in ctx.res.stack.iter_mut().rev() {
        if let EventData::CardUsed(ref mut u) = f.data {
            u.targets.clear();
            u.cancelled = true;
            break;
        }
    }
}

pub(super) fn current_card_use_fid(ctx: &Ctx<'_>) -> Option<EventId> {
    ctx.res.stack.iter().rev().find_map(|f| match f.data {
        EventData::CardUsed(_) => Some(f.id),
        _ => None,
    })
}

pub(super) fn damage_root_card_use_fid(ctx: &Ctx<'_>) -> Option<EventId> {
    let mut damage_idx = None;
    for (idx, f) in ctx.res.stack.iter().enumerate().rev() {
        if matches!(f.data, EventData::Damage(_) | EventData::DamageWindow { .. }) {
            damage_idx = Some(idx);
            break;
        }
    }
    let idx = damage_idx?;
    ctx.res.stack[..idx]
        .iter()
        .rev()
        .find_map(|f| match f.data {
            EventData::CardUsed(_) => Some(f.id),
            _ => None,
        })
}

pub(super) fn damage_effective_kind(
    ctx: &Ctx<'_>,
    d: &crate::event::DamageData,
) -> Option<CardKind> {
    if let Some(root_fid) = damage_root_card_use_fid(ctx) {
        if let Some(f) = ctx.res.frame(root_fid) {
            if let EventData::CardUsed(u) = &f.data {
                return Some(u.effective_kind(ctx.catalog));
            }
        }
    }
    d.card.map(|c| ctx.catalog.resolved_card_kind(ctx.core, c))
}

pub(super) fn distinct_alive_factions(core: &GameStateCore, catalog: &Catalog) -> u32 {
    let mut mask = 0u32;
    for (pid, p) in core.players.iter().enumerate() {
        if !p.alive {
            continue;
        }
        if let Some(g) = p.general_slots[0] {
            if let Some((_, faction, _, _, _)) = catalog.general_meta(g) {
                mask |= 1u32 << (faction as u8);
                continue;
            }
        }
        mask |= 1u32 << ((pid % 3) as u8);
    }
    mask.count_ones()
}

pub(super) fn pop_card_from_deck_or_discard(
    core: &mut GameStateCore,
    pred: impl Fn(CardId) -> bool,
) -> Option<CardId> {
    if let Some(pos) = core.deck.iter().rposition(|&c| pred(c)) {
        return Some(core.deck.remove(pos));
    }
    if let Some(pos) = core.discard.iter().rposition(|&c| pred(c)) {
        return Some(core.discard.remove(pos));
    }
    None
}

pub(super) fn pop_card_from_deck_bottom_or_discard(
    core: &mut GameStateCore,
    pred: impl Fn(CardId) -> bool,
) -> Option<CardId> {
    if let Some(pos) = core.deck.iter().position(|&c| pred(c)) {
        return Some(core.deck.remove(pos));
    }
    if let Some(pos) = core.discard.iter().rposition(|&c| pred(c)) {
        return Some(core.discard.remove(pos));
    }
    None
}

pub(super) fn give_deck_or_discard_card_to(
    ctx: &mut Ctx<'_>,
    to: PlayerId,
    cid: CardId,
) -> bool {
    gain_card_from_deck_or_discard(ctx, to, cid)
}

pub(super) fn pop_detached_or_field_card(
    ctx: &mut Ctx<'_>,
    name: &str,
    skip_equip_owner: Option<PlayerId>,
) -> Option<CardId> {
    if let Some(pos) = ctx
        .core
        .detached_cards
        .iter()
        .position(|&(c, _)| ctx.catalog.cards.get(c as usize).is_some_and(|d| d.name == name))
    {
        return Some(ctx.core.detached_cards.remove(pos).0);
    }
    if let Some(cid) = pop_card_from_deck_or_discard(ctx.core, |c| {
        ctx.catalog.cards.get(c as usize).is_some_and(|d| d.name == name)
    }) {
        return Some(cid);
    }
    if let Some(pos) = ctx
        .core
        .table
        .iter()
        .position(|&c| ctx.catalog.cards.get(c as usize).is_some_and(|d| d.name == name))
    {
        return Some(ctx.core.table.remove(pos));
    }
    let seats = ctx.core.players.len() as PlayerId;
    for seat in 0..seats {
        if skip_equip_owner == Some(seat) {
            continue;
        }
        if let Some(&cid) = ctx
            .core
            .player(seat)
            .equips
            .iter()
            .find(|&&c| ctx.catalog.cards.get(c as usize).is_some_and(|d| d.name == name))
        {
            ctx.take_owned_card(seat, cid);
            ctx.note_cards_lost(seat, &[cid]);
            return Some(cid);
        }
    }
    None
}

pub(super) fn equip_card_onto_player(ctx: &mut Ctx<'_>, target: PlayerId, card: CardId) -> bool {
    let Some(slot) = ctx.catalog.card_slot_of(ctx.core, card) else {
        return false;
    };
    if !crate::rvm_ops::has_enabled_equip_slot(ctx.core, target, slot) {
        return false;
    }
    ctx.core.deck.retain(|&c| c != card);
    ctx.core.discard.retain(|&c| c != card);
    ctx.core.detached_cards.retain(|&(c, _)| c != card);
    for p in 0..ctx.core.players.len() as PlayerId {
        if ctx.core.player(p).hand.contains(&card)
            || ctx.core.player(p).equips.contains(&card)
            || ctx.core.player(p).judge.contains(&card)
        {
            ctx.take_owned_card(p, card);
            ctx.note_cards_lost(p, &[card]);
        }
    }
    if !ctx.core.table.contains(&card) {
        ctx.core.table.push(card);
    }
    ctx.emit(EventData::Equipped {
        player: target,
        card,
        replaced: None,
    });
    true
}

pub(super) fn push_to_pile(core: &mut GameStateCore, owner: PlayerId, pile: &str, cards: &[CardId]) {
    core.vm_aux
        .piles
        .entry((owner, pile.to_string()))
        .or_default()
        .extend(cards.iter().copied());
}

pub(super) fn pile_cards(core: &GameStateCore, owner: PlayerId, pile: &str) -> Vec<CardId> {
    core.vm_aux
        .piles
        .get(&(owner, pile.to_string()))
        .cloned()
        .unwrap_or_default()
}

pub(super) fn take_from_pile(
    core: &mut GameStateCore,
    owner: PlayerId,
    pile: &str,
    cards: &[CardId],
) -> Vec<CardId> {
    let mut taken = Vec::new();
    if let Some(v) = core.vm_aux.piles.get_mut(&(owner, pile.to_string())) {
        for &c in cards {
            if let Some(pos) = v.iter().position(|&x| x == c) {
                v.remove(pos);
                taken.push(c);
            }
        }
        if v.is_empty() {
            core.vm_aux.piles.remove(&(owner, pile.to_string()));
        }
    }
    taken
}

pub(super) fn take_owned_cards_from_zones(
    ctx: &mut Ctx<'_>,
    from: PlayerId,
    cards: &[CardId],
) -> Vec<CardId> {
    let mut taken = Vec::new();
    for &c in cards {
        let owns = ctx.core.player(from).hand.contains(&c)
            || ctx.core.player(from).equips.contains(&c)
            || ctx.core.player(from).judge.contains(&c);
        if owns {
            ctx.take_owned_card(from, c);
            taken.push(c);
        }
    }
    if !taken.is_empty() {
        ctx.note_cards_lost(from, &taken);
    }
    taken
}

pub(super) fn gain_hand_cards_from_pile_or_deck(
    ctx: &mut Ctx<'_>,
    to: PlayerId,
    cards: Vec<CardId>,
) {
    if cards.is_empty() || !ctx.core.player(to).alive {
        return;
    }
    ctx.core.player_mut(to).hand.extend(cards.iter().copied());
    ctx.out.push(LogEvent::CardsDrawn {
        player: to,
        cards: cards.clone(),
    });
    ctx.emit(EventData::CardsDrawn { player: to, cards });
}

pub(super) fn emit_virtual_sha(
    ctx: &mut Ctx<'_>,
    user: PlayerId,
    targets: Vec<PlayerId>,
    no_use_count: bool,
) {
    let valid: Vec<PlayerId> = targets
        .into_iter()
        .filter(|&t| {
            t != user
                && ctx.core.player(t).alive
                && can_target_sha_no_range_npc(ctx.core, ctx.catalog, user, t)
        })
        .collect();
    if valid.is_empty() {
        return;
    }
    if no_use_count {
        set_flag(ctx.core, user, "vm_va_sha_no_count", 1);
    }
    ctx.emit(EventData::CardUsed(card_use_virtual(
        user,
        valid,
        DamageNature::Normal,
        CardKind::Sha,
    )));
}

pub(super) fn emit_virtual_thunder_sha(ctx: &mut Ctx<'_>, user: PlayerId, target: PlayerId) {
    if target == user || !ctx.core.player(target).alive {
        return;
    }
    set_flag(ctx.core, user, "vm_va_sha_no_count", 1);
    ctx.emit(EventData::CardUsed(card_use_virtual(
        user,
        vec![target],
        DamageNature::Thunder,
        CardKind::LeiSha,
    )));
}

pub(super) fn can_target_sha_no_range_npc(
    core: &GameStateCore,
    catalog: &Catalog,
    user: PlayerId,
    target: PlayerId,
) -> bool {
    user != target
        && core.player(target).alive
        && crate::kernel::can_target_sha_no_range(core, catalog, user, target, CardKind::Sha)
        && !crate::rvm_bridge::modifiers::eval_card_target_canceled(
            core,
            catalog,
            user,
            target,
            0,
            CardKind::Sha,
            DamageNature::Normal,
        )
}

pub(super) fn usable_sha_targets_no_range(
    core: &GameStateCore,
    catalog: &Catalog,
    user: PlayerId,
) -> Vec<PlayerId> {
    alive_others(core, user)
        .into_iter()
        .filter(|&t| can_target_sha_no_range_npc(core, catalog, user, t))
        .collect()
}

pub(super) fn card_usable_on_target_with_card(
    core: &GameStateCore,
    catalog: &Catalog,
    user: PlayerId,
    target: PlayerId,
    card: CardId,
    kind: CardKind,
    ignore_distance: bool,
) -> bool {
    if !core.player(target).alive {
        return false;
    }
    if crate::rvm_bridge::modifiers::eval_card_target_canceled(
        core,
        catalog,
        user,
        target,
        card,
        kind,
        DamageNature::Normal,
    ) {
        return false;
    }
    if kind.is_sha_like() {
        if user == target
            || crate::kernel::sha_user_or_target_blocked(core, catalog, user, target)
            || catalog.has_undist(core, target)
        {
            return false;
        }
        return ignore_distance
            || crate::rvm_bridge::modifiers::eval_card_distance_unlimited_to(
                core,
                catalog,
                user,
                Some(target),
                card,
                kind,
                DamageNature::Normal,
            )
            || core.in_attack_range(catalog, user, target);
    }
    match kind {
        CardKind::Snatch => {
            user != target
                && !hej_cards(core, target).is_empty()
                && !catalog.has_undist(core, target)
                && (ignore_distance
                    || crate::rvm_bridge::modifiers::eval_card_distance_unlimited_to(
                        core,
                        catalog,
                        user,
                        Some(target),
                        card,
                        kind,
                        DamageNature::Normal,
                    )
                    || core.computed_distance(catalog, user, target) <= 1)
        }
        CardKind::Dismantle => user != target && !hej_cards(core, target).is_empty(),
        CardKind::Duel => user != target,
        _ if is_huogong_card(catalog, card) => user != target && !core.player(target).hand.is_empty(),
        _ => true,
    }
}

pub(super) fn card_effective_rank_with_mods(
    ctx: &Ctx<'_>,
    owner: PlayerId,
    cid: CardId,
    txyingyang_mode: Option<u32>,
) -> u8 {
    let base = ctx
        .catalog
        .cards
        .get(cid as usize)
        .map_or(1, |d| d.rank);
    let r = if is_unblocked_npc_b(ctx.core, ctx.catalog, owner)
        && has_named_skill(ctx.core, ctx.catalog, owner, "txyingyang")
    {
        super::super::l11_common_b2::apply_yingyang_mode(
            base,
            txyingyang_mode.unwrap_or(2),
        )
    } else {
        base
    };
    r
}

pub(super) fn pindian_resolve_cards(
    ctx: &mut Ctx<'_>,
    p1: PlayerId,
    c1: CardId,
    p2: PlayerId,
    c2: CardId,
) -> (bool, u8, u8) {
    let own_c1 = take_owned_cards_from_zones(ctx, p1, &[c1]);
    let own_c2 = take_owned_cards_from_zones(ctx, p2, &[c2]);
    if !own_c1.is_empty() {
        ctx.to_discard(c1);
        ctx.out.push(LogEvent::CardDiscarded {
            player: p1,
            cards: own_c1,
        });
        ctx.emit(EventData::CardsDiscarded {
            player: p1,
            cards: vec![c1],
        });
    }
    if !own_c2.is_empty() {
        ctx.to_discard(c2);
        ctx.out.push(LogEvent::CardDiscarded {
            player: p2,
            cards: own_c2,
        });
        ctx.emit(EventData::CardsDiscarded {
            player: p2,
            cards: vec![c2],
        });
    }
    let r1 = card_effective_rank_with_mods(ctx, p1, c1, None);
    let r2 = card_effective_rank_with_mods(ctx, p2, c2, None);
    (r1 > r2, r1, r2)
}

#[inline]
pub(super) fn is_unblocked_npc_b(state: &GameStateCore, catalog: &Catalog, owner: PlayerId) -> bool {
    is_unblocked_l11_owner(state, catalog, owner)
}

#[inline]
pub(super) fn is_locked_unblocked_npc_b(
    state: &GameStateCore,
    catalog: &Catalog,
    owner: PlayerId,
) -> bool {
    is_locked_unblocked_l11_owner(state, catalog, owner)
}

pub(super) fn sha_dodged_shan_cards(
    core: &GameStateCore,
    target: PlayerId,
    fid: EventId,
) -> Vec<CardId> {
    let prefix = format!("l11_sha_dodge_ok:{fid}:");
    let mut out: Vec<CardId> = core
        .vm_aux
        .flags
        .keys()
        .filter_map(|(p, k)| {
            if *p == target && k.starts_with(&prefix) {
                k[prefix.len()..].parse::<CardId>().ok()
            } else {
                None
            }
        })
        .filter(|c| core.discard.contains(c) || core.table.contains(c))
        .collect();
    out.sort_unstable();
    out
}

pub(super) fn grant_npc_b_skill(ctx: &mut Ctx<'_>, p: PlayerId, name: &str) {
    ensure_perm_skill(ctx, p, name);
}

pub(super) fn remove_npc_b_skill(ctx: &mut Ctx<'_>, p: PlayerId, name: &str) {
    remove_named_skill(ctx, p, name);
}

pub(super) fn is_weapon_card(catalog: &Catalog, core: &GameStateCore, c: CardId) -> bool {
    catalog.card_slot_of(core, c) == Some(EquipSlot::Weapon)
}

pub(super) fn shefu_kind_to_tag(k: CardKind) -> u8 {
    SHEFU_NAMES.iter().position(|&x| x == k).unwrap_or(0) as u8
}

pub(super) fn shefu_tag_to_kind(t: u8) -> Option<CardKind> {
    SHEFU_NAMES.get(t as usize).copied()
}

pub(super) fn log_skill(ctx: &mut Ctx<'_>, owner: PlayerId, skill: &'static str) {
    ctx.log(LogEvent::SkillTriggered {
        skill,
        owner,
        presentation: None,
    });
}

pub(super) fn can_move_card_to(
    core: &GameStateCore,
    catalog: &Catalog,
    cid: CardId,
    dst: PlayerId,
) -> bool {
    if !core.player(dst).alive {
        return false;
    }
    if let Some(slot) = catalog.card_slot_of(core, cid) {
        return crate::rvm_ops::has_enabled_equip_slot(core, dst, slot)
            && core
                .player(dst)
                .equips
                .iter()
                .all(|&e| catalog.card_slot_of(core, e) != Some(slot));
    }
    let kind = catalog.resolved_card_kind(core, cid);
    !catalog.blocks_delayed_trick(core, dst)
        && core
            .player(dst)
            .judge
            .iter()
            .all(|&j| catalog.resolved_card_kind(core, j) != kind)
}

pub(super) fn move_field_card(
    ctx: &mut Ctx<'_>,
    src: PlayerId,
    cid: CardId,
    dst: PlayerId,
) -> bool {
    if src == dst || !can_move_card_to(ctx.core, ctx.catalog, cid, dst) {
        return false;
    }
    if ctx.core.player(src).equips.contains(&cid) {
        ctx.take_owned_card(src, cid);
        ctx.note_cards_lost(src, &[cid]);
        return equip_card_onto_player(ctx, dst, cid);
    }
    if ctx.core.player(src).judge.contains(&cid) {
        ctx.take_owned_card(src, cid);
        ctx.note_cards_lost(src, &[cid]);
        ctx.core.player_mut(dst).judge.push(cid);
        ctx.emit(EventData::CardsMoved {
            cards: vec![cid],
            from: src,
            to: dst,
        });
        return true;
    }
    false
}

pub(super) fn can_use_basic_in_play(
    core: &GameStateCore,
    catalog: &Catalog,
    owner: PlayerId,
    cid: CardId,
) -> bool {
    let kind = catalog.card_kind(cid);
    if !kind.is_basic_kind() || kind == CardKind::Shan {
        return false;
    }
    if kind == CardKind::Tao {
        return core.player(owner).hp < core.player(owner).max_hp as i32;
    }
    if kind == CardKind::Jiu {
        return !core.turn.jiu_used;
    }
    if kind.is_sha_like() {
        let (limit, unlim) = crate::rvm_bridge::modifiers::eval_sha_use_limit(
            core,
            catalog,
            owner,
            Some(cid),
            kind,
            DamageNature::Normal,
            1,
        );
        let used = flag(core, owner, "l11_phase_sha_count");
        let sha_ok = unlim
            || !core.turn.sha_used_play_phase
            || used < limit;
        if !sha_ok {
            return false;
        }
        return alive_others(core, owner)
            .into_iter()
            .any(|t| card_usable_on_target_with_card(core, catalog, owner, t, cid, kind, false));
    }
    false
}

pub(super) fn same_faction(
    catalog: &Catalog,
    core: &GameStateCore,
    a: PlayerId,
    b: PlayerId,
) -> bool {
    let faction_of = |p: PlayerId| -> Option<crate::catalog::Faction> {
        core.player(p)
            .general_slots[0]
            .and_then(|g| catalog.general_meta(g).map(|m| m.1))
    };
    matches!((faction_of(a), faction_of(b)), (Some(fa), Some(fb)) if fa == fb)
}

pub(super) fn can_use_sha_this_play_phase(
    core: &GameStateCore,
    catalog: &Catalog,
    owner: PlayerId,
) -> bool {
    let (limit, unlim) = crate::rvm_bridge::modifiers::eval_sha_use_limit(
        core,
        catalog,
        owner,
        None,
        CardKind::Sha,
        DamageNature::Normal,
        1,
    );
    let used = flag(core, owner, "l11_phase_sha_count");
    let sha_ok = unlim || !core.turn.sha_used_play_phase || used < limit;
    sha_ok
        && alive_others(core, owner).into_iter().any(|t| {
            core.in_attack_range(catalog, owner, t) && can_target_sha_no_range_npc(core, catalog, owner, t)
        })
}

pub(super) fn re_exports_check(
    _ctx: &mut Ctx<'_>,
    _p: PlayerId,
    _cards: &[CardId],
) -> (usize, bool) {
    // Keeps shared re-exports used cleanly when split across submodules.
    let _ = (
        alive_all,
        discard_player_cards,
        emit_damage,
        emit_lose_hp,
        has_equip_named,
        has_named_skill,
        he_cards,
        hej_cards,
        is_ordinary_trick_kind,
        move_cards_to_hand,
        damage_data_with,
    );
    (0, false)
}
