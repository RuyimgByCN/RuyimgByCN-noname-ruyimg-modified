//! Explicit `L11 太虚幻境` cross-package NPC skills Part B modifier rules (`d34_l11_mods_npc_b.rs`).

use crate::catalog::{CardKind, Catalog, EquipSlot, Suit};
use crate::event::{DamageNature, EventData, Phase};
use crate::ids::{CardId, EventId, PlayerId};
use crate::state::GameStateCore;

use super::d34_flags::{flag_val, has_ext_skill, is_target_unequip2};

#[inline]
pub fn turn_stamp(state: &GameStateCore) -> u32 {
    state
        .turn
        .round
        .saturating_mul(64)
        .saturating_add(u32::from(state.turn.current) + 1)
}

#[inline]
pub fn phase_stamp(state: &GameStateCore) -> u32 {
    turn_stamp(state)
        .saturating_mul(8)
        .saturating_add((state.turn.phase as u32) + 1)
}

#[inline]
pub fn suit_bit(suit: Suit) -> u32 {
    match suit {
        Suit::Spade => 1,
        Suit::Heart => 2,
        Suit::Club => 4,
        Suit::Diamond => 8,
    }
}

pub fn has_equipped_named_npc_b(
    state: &GameStateCore,
    catalog: &Catalog,
    seat: PlayerId,
    name: &str,
) -> bool {
    state
        .player(seat)
        .equips
        .iter()
        .any(|&c| catalog.cards.get(c as usize).is_some_and(|cd| cd.name == name))
}

#[inline]
pub fn is_red_suit(suit: Suit) -> bool {
    matches!(suit, Suit::Heart | Suit::Diamond)
}

pub fn apply_hand_limit_npc_b(
    state: &GameStateCore,
    catalog: &Catalog,
    player: PlayerId,
    mut limit: i32,
) -> i32 {
    let tstamp = turn_stamp(state);
    if flag_val(state, player, "l11b_sbyingzi_turn_stamp") == tstamp {
        limit += flag_val(state, player, "l11b_sbyingzi_hand_bonus") as i32;
    }
    if flag_val(state, player, "l11b_shunshi_stacks") > 0 {
        limit += flag_val(state, player, "l11b_shunshi_stacks") as i32;
    }
    if flag_val(state, player, "l11b_twmibei_less_stamp") == tstamp {
        limit -= 1;
    }
    if flag_val(state, player, "l11b_wangzun_less") > 0 {
        limit -= 1;
    }
    // `taipingyaoshu` (`guozhan.js:1962`): hand limit + (distinct alive factions - 1),
    // skipping if `th_taipingyaoshu` already added that bonus in `d34_flags::apply_hand_limit`.
    let already_th_tpys = flag_val(state, player, "th_taipingyaoshu_prevent_elemental") > 0
        || flag_val(state, player, "l28_taipingyaoshu_equipped") > 0
        || has_equipped_named_npc_b(state, catalog, player, "th_taipingyaoshu")
        || has_ext_skill(state, catalog, player, "th_taipingyaoshu");
    let has_tpys = has_equipped_named_npc_b(state, catalog, player, "taipingyaoshu")
        || has_ext_skill(state, catalog, player, "taipingyaoshu");
    if !already_th_tpys && has_tpys && !is_target_unequip2(state, player) {
        let mut faction_mask = 0u16;
        for (pid, p) in state.players.iter().enumerate() {
            if !p.alive {
                continue;
            }
            if let Some(g) = p.general_slots[0] {
                if let Some((_, faction, _, _, _)) = catalog.general_meta(g) {
                    faction_mask |= 1u16 << (faction as u8);
                    continue;
                }
            }
            faction_mask |= 1u16 << ((pid % 3) as u8);
        }
        let distinct = faction_mask.count_ones() as i32;
        limit += (distinct - 1).max(0);
    }
    limit
}

pub fn apply_attack_range_npc_b(state: &GameStateCore, player: PlayerId, mut cur: i32) -> i32 {
    let tstamp = turn_stamp(state);
    if flag_val(state, player, "l11b_twxiangyu_stamp") == tstamp {
        cur += flag_val(state, player, "l11b_twxiangyu_mask").count_ones() as i32;
    }
    cur
}

pub fn apply_sha_use_limit_npc_b(
    state: &GameStateCore,
    catalog: &Catalog,
    player: PlayerId,
    card: Option<CardId>,
    kind: CardKind,
    mut cur: i32,
    mut unlimited: bool,
) -> (i32, bool) {
    let tstamp = turn_stamp(state);
    let pstamp = phase_stamp(state);
    if flag_val(state, player, "l11b_shunshi_stacks") > 0 {
        cur += flag_val(state, player, "l11b_shunshi_stacks") as i32;
    }
    if flag_val(state, player, "l11b_tuifeng_turn_stamp") == tstamp {
        cur += flag_val(state, player, "l11b_tuifeng_sha_bonus") as i32;
    }
    if flag_val(state, player, "l11b_twyilie_phase_stamp") == pstamp
        && flag_val(state, player, "l11b_twyilie_opt1") > 0
    {
        cur += 1;
    }
    if kind.is_sha_like() && has_ext_skill(state, catalog, player, "twwushen") {
        if let Some(c) = card {
            if (c as usize) < catalog.cards.len()
                && catalog.effective_card_suit(state, player, c) == Suit::Heart
            {
                unlimited = true;
            }
        }
    }
    (cur, unlimited)
}

pub fn apply_card_distance_unlimited_npc_b(
    state: &GameStateCore,
    catalog: &Catalog,
    player: PlayerId,
    card: CardId,
    kind: CardKind,
    mut unlimited: bool,
) -> bool {
    let pstamp = phase_stamp(state);
    if flag_val(state, player, "l11b_zhanyi_trick_stamp") == pstamp {
        unlimited = true;
    }
    if flag_val(state, player, "l11b_twxingqi_range") > 0 {
        unlimited = true;
    }
    if flag_val(state, player, &format!("l11b_twqirang_card:{card}")) > 0 {
        unlimited = true;
    }
    if kind.is_sha_like()
        && has_ext_skill(state, catalog, player, "twwushen")
        && (card as usize) < catalog.cards.len()
        && catalog.effective_card_suit(state, player, card) == Suit::Heart
    {
        unlimited = true;
    }
    unlimited
}

pub fn apply_sha_nature_npc_b(
    state: &GameStateCore,
    catalog: &Catalog,
    actor: PlayerId,
    card: Option<CardId>,
    kind: CardKind,
    nature: DamageNature,
) -> DamageNature {
    if !kind.is_sha_like() {
        return nature;
    }
    if has_ext_skill(state, catalog, actor, "sbliegong")
        && !state
            .player(actor)
            .equips
            .iter()
            .any(|&c| catalog.card_slot_of(state, c) == Some(EquipSlot::Weapon))
    {
        return DamageNature::Normal;
    }
    if has_ext_skill(state, catalog, actor, "twwushen") {
        if let Some(c) = card {
            if (c as usize) < catalog.cards.len()
                && catalog.effective_card_suit(state, actor, c) == Suit::Heart
            {
                return DamageNature::Normal;
            }
        }
    }
    nature
}

pub fn apply_card_target_canceled_npc_b(
    state: &GameStateCore,
    actor: PlayerId,
    target: PlayerId,
    mut canceled: bool,
) -> bool {
    let tstamp = turn_stamp(state);
    if target != actor && flag_val(state, actor, "l11b_zhidao2_turn_stamp") == tstamp {
        canceled = true;
    }
    canceled
}

pub fn is_card_ignored_hand_limit_npc_b(
    state: &GameStateCore,
    catalog: &Catalog,
    player: PlayerId,
    card: CardId,
) -> bool {
    if (card as usize) >= catalog.cards.len() {
        return false;
    }
    has_ext_skill(state, catalog, player, "xiehui")
        && !is_red_suit(catalog.effective_card_suit(state, player, card))
}

pub fn is_card_ignored_in_discard_phase_npc_b(
    state: &GameStateCore,
    catalog: &Catalog,
    player: PlayerId,
    card: CardId,
) -> bool {
    if (card as usize) >= catalog.cards.len() {
        return false;
    }
    let in_hand = state.player(player).hand.contains(&card);
    if !in_hand {
        return false;
    }
    if flag_val(state, player, &format!("l11b_xiehui2_card:{card}")) > 0 {
        return true;
    }
    if has_ext_skill(state, catalog, player, "twyuhua")
        && !catalog.resolved_card_kind(state, card).is_basic_kind()
    {
        return true;
    }
    if has_ext_skill(state, catalog, player, "xiehui")
        && !is_red_suit(catalog.effective_card_suit(state, player, card))
    {
        return true;
    }
    false
}

pub fn is_card_forbidden_npc_b(
    state: &GameStateCore,
    catalog: &Catalog,
    actor: PlayerId,
    card: CardId,
) -> bool {
    if (card as usize) >= catalog.cards.len() || !state.player(actor).hand.contains(&card) {
        return false;
    }
    if flag_val(state, actor, &format!("l11b_xiehui2_card:{card}")) > 0 {
        return true;
    }
    let is_red = is_red_suit(catalog.effective_card_suit(state, actor, card));
    let tstamp = turn_stamp(state);
    if flag_val(state, actor, "l11b_twqianxi2_stamp") == tstamp {
        let forbidden_color = flag_val(state, actor, "l11b_twqianxi2_color");
        if (forbidden_color == 1 && is_red) || (forbidden_color == 2 && !is_red) {
            return true;
        }
    }
    let qx3 = flag_val(state, actor, "l11b_twqianxi3_blocked_color");
    if (qx3 == 1 && is_red) || (qx3 == 2 && !is_red) {
        return true;
    }
    false
}

pub fn apply_card_unrespondable_npc_b(
    state: &GameStateCore,
    catalog: &Catalog,
    source: PlayerId,
    target: PlayerId,
    card: CardId,
    kind: CardKind,
    mut unrespondable: bool,
) -> bool {
    let is_trick = kind.is_ordinary_trick_kind();
    if (kind.is_sha_like() || is_trick) && has_ext_skill(state, catalog, source, "twsaotao") {
        unrespondable = true;
    }
    if target != source {
        let tstamp = turn_stamp(state);
        if flag_val(state, source, "l11b_spwenji_stamp") == tstamp
            && flag_val(state, source, "l11b_spwenji_kind") == (kind as u32) + 1
        {
            unrespondable = true;
        }
    }
    if flag_val(state, source, &format!("l11b_twqirang_card:{card}")) > 0 {
        unrespondable = true;
    }
    if flag_val(state, source, "l11b_sha_direct_hit_cur") > 0 && kind.is_sha_like() {
        unrespondable = true;
    }
    unrespondable
}

pub fn tpys_damage_prevented_npc_b(
    state: &GameStateCore,
    catalog: &Catalog,
    source: Option<PlayerId>,
    target: PlayerId,
    card: Option<CardId>,
    nature: DamageNature,
) -> bool {
    if nature == DamageNature::Normal || is_target_unequip2(state, target) {
        return false;
    }
    if source.is_some_and(|src| {
        super::d34_l28_mods::source_ignores_armor(state, catalog, src, target, card, None, nature)
    }) {
        return false;
    }
    has_equipped_named_npc_b(state, catalog, target, "taipingyaoshu")
        || has_ext_skill(state, catalog, target, "taipingyaoshu")
}

pub fn is_sbliegong_blocked_shan(
    state: &GameStateCore,
    catalog: &Catalog,
    responder: PlayerId,
    card: CardId,
    active_sha_fid: Option<EventId>,
) -> bool {
    if (card as usize) >= catalog.cards.len() {
        return false;
    }
    let Some(fid) = active_sha_fid else {
        return false;
    };
    let mask = flag_val(state, responder, &format!("l11b_sbliegong_block_mask:{fid}"));
    if mask == 0 {
        return false;
    }
    let bit = suit_bit(catalog.effective_card_suit(state, responder, card));
    (mask & bit) != 0
}

pub fn sha_extra_shan_for_fid(
    state: &GameStateCore,
    target: PlayerId,
    active_sha_fid: Option<EventId>,
) -> u8 {
    let Some(fid) = active_sha_fid else {
        return 0;
    };
    flag_val(state, target, &format!("l11b_twxiangyu_extra_shan:{fid}")) as u8
}

/// Active-frame helper: resolve current `CardUsed` frame ID from resolution stack.
pub fn stack_active_card_use_fid(stack: &[crate::event::Frame]) -> Option<EventId> {
    stack.iter().rev().find_map(|f| match f.data {
        EventData::CardUsed(_) => Some(f.id),
        _ => None,
    })
}

/// `Phase::Play` check used by modifiers without direct cycle.
pub fn is_own_play_phase(state: &GameStateCore, p: PlayerId) -> bool {
    state.turn.current == p && state.turn.phase == Phase::Play
}
