//! Deterministic fallback target and card selector for `commit_up37`.
//!
//! Reads `ctx.res.answer` if a pre-supplied answer is present in the resolution context,
//! and otherwise selects targets/cards automatically using public-information attitude
//! heuristics and card tactical priority.
//! Note: This module does NOT open interactive player choices (`open_choice`);
//! skills requiring genuine player interaction must be routed through `HostBehavior` + `HostStage`.

use crate::choice::ChoiceOption;
use crate::ids::{CardId, PlayerId};
use crate::machine::Ctx;
use crate::state::GameStateCore;
use super::commit::discard_some;

/// Calculates public-information attitude from `from` toward `to`.
/// Positive indicates ally/friendly; negative indicates enemy/hostile; `0` indicates neutral/unknown.
///
/// Information isolation invariant:
/// In identity mode, only the Lord (`core.is_lord(seat)`) is publicly known while alive.
/// Unrevealed non-Lord seats (`Loyalist`, `Rebel`, `Traitor`) are NEVER inspected as targets
/// and are strictly treated as neutral (`0`).
pub fn get_upstream_attitude(core: &GameStateCore, from: PlayerId, to: PlayerId) -> i32 {
    if from == to {
        return 10;
    }
    // In 2-player or boss matches, the single other side is publicly the opponent
    if core.players.len() <= 2 {
        return -10;
    }
    match &core.mode {
        crate::state::GameMode::Identity { roles } => {
            // Only the Lord's identity is publicly revealed on the tabletop.
            // If `to` is not the public Lord, never inspect `to`'s secret role; treat as neutral (0).
            if core.is_lord(to) {
                match roles.get(from as usize) {
                    Some(crate::state::Role::Loyalist) => 8,
                    Some(crate::state::Role::Rebel) => -10,
                    Some(crate::state::Role::Traitor) => 0,
                    Some(crate::state::Role::Lord) => 10,
                    None => 0,
                }
            } else {
                0
            }
        }
        crate::state::GameMode::Kingdoms => {
            let f_from = &core.player(from).kingdom_identity;
            let f_to = &core.player(to).kingdom_identity;
            match (f_from, f_to) {
                (
                    crate::state::KingdomIdentity::Declared(a),
                    crate::state::KingdomIdentity::Declared(b),
                ) => {
                    if a == b {
                        8
                    } else {
                        -8
                    }
                }
                (
                    crate::state::KingdomIdentity::Declared(_),
                    crate::state::KingdomIdentity::Ye,
                )
                | (
                    crate::state::KingdomIdentity::Ye,
                    crate::state::KingdomIdentity::Declared(_) | crate::state::KingdomIdentity::Ye,
                ) => -6,
                // Unrevealed (`KingdomIdentity::Unknown`) seats are treated as neutral (0)
                _ => 0,
            }
        }
        _ => -5,
    }
}

/// Deterministic fallback selector for a hostile target: reads `ctx.res.answer` if present,
/// otherwise picks the candidate with lowest attitude and lowest HP (does NOT call `open_choice`).
pub fn hostile_target(ctx: &mut Ctx<'_>, owner: PlayerId) -> Option<PlayerId> {
    filtered_target(ctx, owner, true, |_, _| true)
}

/// Deterministic fallback selector for a friendly target: reads `ctx.res.answer` if present,
/// otherwise picks the candidate with highest attitude and lowest HP (does NOT call `open_choice`).
pub fn friendly_target(ctx: &mut Ctx<'_>, owner: PlayerId) -> Option<PlayerId> {
    filtered_target(ctx, owner, false, |_, _| true)
}

/// Deterministic fallback selector for a candidate matching `filter`: reads `ctx.res.answer`
/// if present, otherwise sorts eligible candidates by public attitude and HP (does NOT call `open_choice`).
pub fn filtered_target<F>(
    ctx: &mut Ctx<'_>,
    owner: PlayerId,
    hostile: bool,
    filter: F,
) -> Option<PlayerId>
where
    F: Fn(&GameStateCore, PlayerId) -> bool,
{
    // 1. Check if an answer from player is present in resolution
    if let Some(ans) = ctx.res.answer.as_ref() {
        for opt in ans {
            if let ChoiceOption::Player(p) = opt {
                if *p != owner && ctx.core.player(*p).alive && filter(ctx.core, *p) {
                    return Some(*p);
                }
            }
        }
    }

    // 2. Collect eligible candidates
    let mut candidates: Vec<PlayerId> = ctx
        .core
        .alive_players()
        .filter(|&p| p != owner && filter(ctx.core, p))
        .collect();
    if candidates.is_empty() {
        return None;
    }

    // 3. Sort candidates according to attitude and tactical value
    if hostile {
        // Most hostile first (lowest attitude), then lowest HP (kill priority)
        candidates.sort_by(|&a, &b| {
            let att_a = get_upstream_attitude(ctx.core, owner, a);
            let att_b = get_upstream_attitude(ctx.core, owner, b);
            att_a
                .cmp(&att_b)
                .then_with(|| ctx.core.player(a).hp.cmp(&ctx.core.player(b).hp))
        });
    } else {
        // Most friendly first (highest attitude), then lowest HP (injured ally rescue)
        candidates.sort_by(|&a, &b| {
            let att_a = get_upstream_attitude(ctx.core, owner, a);
            let att_b = get_upstream_attitude(ctx.core, owner, b);
            att_b
                .cmp(&att_a)
                .then_with(|| ctx.core.player(a).hp.cmp(&ctx.core.player(b).hp))
        });
    }

    // 4. Return top evaluated candidate
    candidates.into_iter().next()
}

/// Chooses a card from own hand or equip according to strategic priority
/// (avoiding peach when wounded, preferring non-defense cards).
pub fn choose_own_card(
    ctx: &mut Ctx<'_>,
    owner: PlayerId,
    allow_equip: bool,
) -> Option<CardId> {
    // Check for answered card
    if let Some(ans) = ctx.res.answer.as_ref() {
        for opt in ans {
            if let ChoiceOption::Card(c) = opt {
                let in_hand = ctx.core.player(owner).hand.contains(c);
                let in_equip = allow_equip && ctx.core.player(owner).equips.contains(c);
                if in_hand || in_equip {
                    return Some(*c);
                }
            }
        }
    }

    let wounded = ctx.core.player(owner).hp < ctx.core.player(owner).max_hp as i32;

    // Search hand cards by tactical priority: Peach preserved if wounded, Shan preserved
    let best_hand = ctx.core.player(owner).hand.iter().copied().min_by_key(|&cid| {
        let def = ctx.catalog.cards.get(cid as usize);
        let kind = def.map(|d| d.kind);
        match kind {
            Some(crate::catalog::CardKind::Tao) if wounded => 4,
            Some(crate::catalog::CardKind::Shan) => 3,
            Some(crate::catalog::CardKind::Sha | crate::catalog::CardKind::FireSha) => 2,
            _ => 1,
        }
    });
    if let Some(c) = best_hand {
        return Some(c);
    }

    // Search equip cards by defensive loss: prefer discarding lower defensive value first
    if allow_equip && !ctx.core.player(owner).equips.is_empty() {
        return ctx.core.player(owner).equips.iter().copied().min_by_key(|&cid| {
            let def = ctx.catalog.cards.get(cid as usize);
            match def.map(|d| d.kind) {
                Some(crate::catalog::CardKind::Armor) => 4,
                Some(crate::catalog::CardKind::HorsePlus) => 3,
                Some(crate::catalog::CardKind::Weapon) => 2,
                Some(crate::catalog::CardKind::HorseMinus) => 1,
                _ => 0,
            }
        });
    }
    None
}

/// Chooses a card from an enemy's hand or equip according to strategic priority
/// (preferring armor or weapons of high tactical impact).
pub fn choose_enemy_card(
    ctx: &mut Ctx<'_>,
    _owner: PlayerId,
    target: PlayerId,
) -> Option<CardId> {
    choose_enemy_card_opt(ctx, _owner, target, true)
}

pub fn choose_enemy_card_opt(
    ctx: &mut Ctx<'_>,
    _owner: PlayerId,
    target: PlayerId,
    allow_equip: bool,
) -> Option<CardId> {
    // Check for answered card
    if let Some(ans) = ctx.res.answer.as_ref() {
        for opt in ans {
            if let ChoiceOption::Card(c) = opt {
                let in_hand = ctx.core.player(target).hand.contains(c);
                let in_equip = allow_equip && ctx.core.player(target).equips.contains(c);
                if in_hand || in_equip {
                    return Some(*c);
                }
            }
        }
    }

    // Prefer stripping high-threat equipment first (armor > +1 horse > weapon > -1 horse)
    if allow_equip && !ctx.core.player(target).equips.is_empty() {
        return ctx.core.player(target).equips.iter().copied().max_by_key(|&cid| {
            let def = ctx.catalog.cards.get(cid as usize);
            match def.map(|d| d.kind) {
                Some(crate::catalog::CardKind::Armor) => 4,
                Some(crate::catalog::CardKind::HorsePlus) => 3,
                Some(crate::catalog::CardKind::Weapon) => 2,
                Some(crate::catalog::CardKind::HorseMinus) => 1,
                _ => 0,
            }
        });
    }

    // Otherwise take hand card at random from concealed hand via deterministic RNG
    let hand_len = ctx.core.player(target).hand.len();
    if hand_len == 0 {
        None
    } else {
        let idx = ctx.core.rng.gen_range(hand_len as u32) as usize;
        ctx.core.player(target).hand.get(idx).copied()
    }
}

/// Discards one card from `seat`'s hand or equips.
pub fn discard_one_he(ctx: &mut Ctx<'_>, seat: PlayerId) -> Option<CardId> {
    discard_one_he_opt(ctx, seat, true)
}

pub fn discard_one_he_opt(ctx: &mut Ctx<'_>, seat: PlayerId, allow_equip: bool) -> Option<CardId> {
    let card = choose_own_card(ctx, seat, allow_equip)?;
    if ctx.core.player(seat).hand.contains(&card) {
        ctx.take_hand_card(seat, card);
        discard_some(ctx, seat, vec![card]);
        Some(card)
    } else if allow_equip && ctx.core.player(seat).equips.contains(&card) {
        ctx.take_owned_card(seat, card);
        discard_some(ctx, seat, vec![card]);
        Some(card)
    } else {
        None
    }
}

/// Discards one card from target `t` by owner (e.g. dismantling).
pub fn discard_target_he(ctx: &mut Ctx<'_>, owner: PlayerId, t: PlayerId) -> Option<CardId> {
    discard_target_he_opt(ctx, owner, t, true)
}

pub fn discard_target_he_opt(ctx: &mut Ctx<'_>, owner: PlayerId, t: PlayerId, allow_equip: bool) -> Option<CardId> {
    let card = choose_enemy_card_opt(ctx, owner, t, allow_equip)?;
    if ctx.core.player(t).hand.contains(&card) {
        ctx.take_hand_card(t, card);
        discard_some(ctx, t, vec![card]);
        Some(card)
    } else if allow_equip && ctx.core.player(t).equips.contains(&card) {
        ctx.take_owned_card(t, card);
        discard_some(ctx, t, vec![card]);
        Some(card)
    } else {
        None
    }
}

/// Steals one card from target `t` into `owner`'s hand.
pub fn steal_target_he(ctx: &mut Ctx<'_>, owner: PlayerId, t: PlayerId) -> bool {
    steal_target_he_opt(ctx, owner, t, true).is_some()
}

pub fn steal_target_he_opt(ctx: &mut Ctx<'_>, owner: PlayerId, t: PlayerId, allow_equip: bool) -> Option<CardId> {
    let card = choose_enemy_card_opt(ctx, owner, t, allow_equip)?;
    if ctx.core.player(t).hand.contains(&card) {
        ctx.take_hand_card(t, card);
        ctx.core.player_mut(owner).hand.push(card);
        Some(card)
    } else if allow_equip && ctx.core.player(t).equips.contains(&card) {
        ctx.take_owned_card(t, card);
        ctx.core.player_mut(owner).hand.push(card);
        Some(card)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event::Phase;
    use crate::rng::RuleRng;
    use crate::state::{GameMode, KingdomIdentity, PlayerState, Role, TurnState, VmAux};
    use std::collections::BTreeMap;

    fn sample_identity_core(roles: Vec<Role>) -> GameStateCore {
        let n = roles.len();
        GameStateCore {
            revision: 1,
            ruleset_version: 333,
            mode: GameMode::Identity { roles },
            players: (0..n).map(|_| PlayerState::new(4)).collect(),
            seat_ring: (0..n as u8).collect(),
            deck: Vec::new(),
            discard: Vec::new(),
            detached_cards: Vec::new(),
            table: Vec::new(),
            rng: RuleRng::from_seed(42),
            turn: TurnState {
                current: 0,
                phase: Phase::Play,
                round: 1,
                sha_used_play_phase: false,
                mufeng_used_basic: false,
                wangchen_discard_basic: false,
                shenmiguo_used: false,
                shuchui_armed: false,
                shuchui_hit: false,
                lianda_used: false,
                land_used: false,
                land_sha_used: false,
                discard_phase_discarded: 0,
                ciqiu_dying: Vec::new(),
                kangyin2: None,
                huanxia_track: Vec::new(),
                cards_used_play_phase: 0,
                cards_used_turn: 0,
                liaoyuan_count: 0,
                xuanyuanjian_marked: false,
                kunlunjing_snapshot: Vec::new(),
                kunlunjing_hp: 0,
                extra_phases: Vec::new(),
                extra_phase_lingwu: false,
                bte_used_play_phase: 0,
                phase_number: 1,
                rende_given: 0,
                jiu_used: false,
                mujiaren_used: false,
                jiu_active: false,
                turn_opened: true,
                skip_draw_phase: false,
                skip_play_phase: false,
                guxing_armed: false,
                guxing_count: 0,
                mengtai_draw_pending: false,
                mengtai_use_pending: false,
                zhufangshenshi_target: None,
                yuedong_num: 0,
                yuedong_eff: 0,
                yuedong_recover: false,
                liangou_hook: None,
                liangou_bonus_used: false,
                feitiao_target: None,
                fanling_used: false,
                diewu_draw_used: false,
                lueying_used: false,
                txianqu_used: false,
                jtjubao_used: false,
                zhuyue_targets: Vec::new(),
                sheling_used: 0,
                sheying_used: false,
                zhangmu_used: false,
                yingfeng_chain: false,
                luanjian_card: None,
                minwan_targets: Vec::new(),
                yanyu_count: 0,
                xjyufeng_used: 0,
                qizuo_copy: None,
                jidian_chain: false,
            },
            skills: BTreeMap::new(),
            declared_kinds: BTreeMap::new(),
            declared_slots: BTreeMap::new(),
            card_natures: BTreeMap::new(),
            current_land: None,
            outcome: None,
            vm_aux: VmAux::default(),
        }
    }

    #[test]
    fn upstream_attitude_never_leaks_unrevealed_identity() {
        for secret_role in [Role::Rebel, Role::Loyalist, Role::Traitor] {
            let core = sample_identity_core(vec![
                Role::Lord,
                Role::Loyalist,
                secret_role,
                Role::Rebel,
                Role::Traitor,
            ]);
            // Any observer looking at unrevealed seat 2 must see identical neutral attitude (0),
            // regardless of whether seat 2 is secretly Rebel, Loyalist, or Traitor.
            for observer in [0u8, 1, 3, 4] {
                assert_eq!(
                    get_upstream_attitude(&core, observer, 2),
                    0,
                    "observer {observer} must treat unrevealed seat 2 ({secret_role:?}) as neutral (0)"
                );
            }
            // Meanwhile, the publicly revealed Lord (seat 0) has role-dependent attitude from each observer:
            assert_eq!(get_upstream_attitude(&core, 1, 0), 8);
            assert_eq!(get_upstream_attitude(&core, 3, 0), -10);
            assert_eq!(get_upstream_attitude(&core, 4, 0), 0);
        }
    }

    #[test]
    fn upstream_attitude_kingdoms_treats_unknown_as_neutral() {
        let mut core = sample_identity_core(vec![Role::Lord, Role::Rebel, Role::Traitor]);
        core.mode = GameMode::Kingdoms;
        core.player_mut(0).kingdom_identity = KingdomIdentity::Declared("wei".to_string());
        core.player_mut(1).kingdom_identity = KingdomIdentity::Unknown;
        core.player_mut(2).kingdom_identity = KingdomIdentity::Declared("shu".to_string());

        assert_eq!(get_upstream_attitude(&core, 0, 1), 0);
        assert_eq!(get_upstream_attitude(&core, 1, 0), 0);
        assert_eq!(get_upstream_attitude(&core, 0, 2), -8);
    }
}
