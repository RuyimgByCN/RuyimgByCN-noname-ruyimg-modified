#![allow(dead_code)]

pub use super::support::*;

pub mod parity_a;
pub mod parity_b;
pub mod shared_vm;

use engine_native::catalog::{CardKind, Suit};
use engine_native::content::host::HostBehavior;
use engine_native::ids::{CardId, PlayerId};
use engine_native::{Engine, GameAction};
use content_native_loader::{start_locked_engine, LockedStartMode};

pub fn start_3p(seed: u64) -> Engine {
    let mut e = start_locked_engine(seed, 3, LockedStartMode::Prototype, taixu_verified())
        .expect("start 3p locked taixu");
    e.debug_sweep_judge_zones();
    e
}

pub fn place_card_suit_rank(
    e: &mut Engine,
    seat: PlayerId,
    suit: Suit,
    kind: CardKind,
    min_rank: u8,
    max_rank: u8,
) -> CardId {
    let c = take_deck_or_discard_by(e, |d| {
        d.kind == kind && d.suit == suit && d.rank >= min_rank && d.rank <= max_rank
    });
    e.state_core_mut().player_mut(seat).hand.push(c);
    c
}

pub fn place_hand_card_named(e: &mut Engine, seat: PlayerId, name: &str) -> CardId {
    let c = take_deck_or_discard_by(e, |d| d.name == name);
    e.state_core_mut().player_mut(seat).hand.push(c);
    c
}

pub fn place_suit_rank_any(
    e: &mut Engine,
    seat: PlayerId,
    suit: Suit,
    min_rank: u8,
    max_rank: u8,
) -> CardId {
    let c = take_deck_or_discard_by(e, |d| {
        d.suit == suit && d.rank >= min_rank && d.rank <= max_rank
    });
    e.state_core_mut().player_mut(seat).hand.push(c);
    c
}

pub fn equip_kind_suit(
    e: &mut Engine,
    seat: PlayerId,
    kind: CardKind,
    suit: Option<Suit>,
) -> CardId {
    let c = take_deck_or_discard_by(e, |d| {
        d.kind == kind && suit.is_none_or(|s| d.suit == s)
    });
    let equip_skills: Vec<&'static str> = e.catalog().cards[c as usize].equip_skills.to_vec();
    e.state_core_mut().player_mut(seat).equips.push(c);
    for sk in equip_skills {
        e.debug_grant(seat, sk);
    }
    c
}

pub fn equip_weapon_range(e: &mut Engine, seat: PlayerId, min_range: u32) -> CardId {
    let c = take_deck_or_discard_by(e, |d| {
        d.kind == CardKind::Weapon && d.range >= min_range
    });
    let equip_skills: Vec<&'static str> = e.catalog().cards[c as usize].equip_skills.to_vec();
    e.state_core_mut().player_mut(seat).equips.push(c);
    for sk in equip_skills {
        e.debug_grant(seat, sk);
    }
    c
}

pub fn move_to_deck_top_suit_rank(e: &mut Engine, suit: Suit, min_rank: u8, max_rank: u8) -> CardId {
    let c = take_deck_or_discard_by(e, |d| {
        d.suit == suit && d.rank >= min_rank && d.rank <= max_rank
    });
    e.state_core_mut().deck.push(c);
    c
}

pub fn pile(e: &Engine, seat: PlayerId, name: &str) -> Vec<CardId> {
    e.state_core()
        .vm_aux
        .piles
        .get(&(seat, name.to_string()))
        .cloned()
        .unwrap_or_default()
}

pub fn assert_is_vm_c(e: &Engine, names: &[&str]) {
    for &sk in names {
        let def = e
            .catalog()
            .skill_by_name(sk)
            .unwrap_or_else(|| panic!("missing skill {sk}"));
        assert_eq!(
            e.catalog().skills[def as usize].behavior.as_host_behavior(),
            Some(HostBehavior::L11VmFixC),
            "skill {sk} must bind HostBehavior::L11VmFixC"
        );
    }
}

pub fn plan_ai_answer(e: &Engine, actor: PlayerId) -> GameAction {
    let ai = engine_native::ai::BoundedPlanner::new(16);
    e.plan_action(actor, &ai).action
}
