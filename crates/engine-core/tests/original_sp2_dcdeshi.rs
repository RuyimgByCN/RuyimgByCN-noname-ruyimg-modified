//! Golden for SP2「得势」(dcdeshi): while wounded, Sha damage to the owner is
//! prevented, the owner gains a Sha from the draw/discard piles, and the owner
//! loses 1 max HP.

use engine_api::{
    CardDefId, CardId, Command, CommandKind, CommandResult, DomainEvent, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::{damage_nature, EffectRequest};

const DCDESHI: u32 = 4_143_976_690;
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/sp2/sp2.npack.json");

fn package() -> content_schema::ContentPackage {
    content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap()
}

fn setup(seed: u64) -> (Transaction, PlayerId, PlayerId) {
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package()).unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 2, seed, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let (owner, source) = (players[0], players[1]);
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.extend(players.iter().map(|player| EffectRequest::DiscardFromHand {
        player: *player,
        count: u32::MAX,
    }));
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().revision = 1;
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    (tx, owner, source)
}

fn add_skill(tx: &mut Transaction, player: PlayerId, def: u32) {
    run_effects(
        tx,
        vec![EffectRequest::AddSkill {
            player,
            skill_def: SkillDefId::new(def),
        }],
    )
    .unwrap();
}

fn hand_sha_kind(tx: &Transaction, player: PlayerId) -> Vec<CardKind> {
    tx.state()
        .hand_cards(player)
        .iter()
        .map(|card| tx.state().cards.get(*card).unwrap().kind)
        .collect()
}

#[test]
fn package_declares_dcdeshi_wounded_sha_damage_before_trigger() {
    let package = package();
    let trigger = package
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == DCDESHI)
        .expect("dcdeshi damage_before trigger");
    assert_eq!(trigger.event, "damage_before");
    assert_eq!(trigger.filter, "owner_is_target_wounded_sha");
    assert!(!trigger.optional);
    assert!(!package
        .modifier_index
        .iter()
        .any(|m| m.skill_def_id == DCDESHI));
}

#[test]
fn wounded_owner_absorbs_sha_damage_gains_sha_and_loses_max_hp() {
    let (mut tx, owner, source) = setup(20_261_201);
    add_skill(&mut tx, owner, DCDESHI);
    // The lord starts at hp = base max (5) while lord bonus sets max 6, so it is
    // already wounded; wound further to keep it unambiguous.
    let hp = tx.state().players.get(owner).unwrap().hp;
    tx.state_mut().players.get_mut(owner).unwrap().hp = hp - 1;
    let max_hp_before = tx.state().players.get(owner).unwrap().max_hp;
    let hand_before = tx.state().hand_count(owner);

    let events = run_effects(
        &mut tx,
        vec![EffectRequest::DealCardDamage {
            source,
            target: owner,
            amount: 1,
            is_lose_hp: false,
            nature: damage_nature::NORMAL,
            card: CardId::NONE,
            card_kind: CardKind::Sha.tag(),
            card_flags: 0,
        }],
    )
    .unwrap();
    // Damage is prevented: no DamageDealt event and HP unchanged.
    assert!(!events.iter().any(|event| matches!(event, DomainEvent::DamageDealt { .. })));
    assert_eq!(
        tx.state().players.get(owner).unwrap().hp,
        hp - 1,
        "sha damage to wounded owner is prevented"
    );
    // Exactly one Sha joined the hand (from draw or discard pile) and max HP dropped by one.
    assert_eq!(tx.state().hand_count(owner), hand_before + 1);
    let gained = tx
        .state()
        .hand_cards(owner)
        .iter()
        .map(|card| tx.state().cards.get(*card).unwrap().kind)
        .collect::<Vec<_>>();
    assert!(gained.contains(&CardKind::Sha));
    assert_eq!(tx.state().players.get(owner).unwrap().max_hp, max_hp_before - 1);
}

#[test]
fn healthy_owner_takes_sha_damage_unchanged() {
    let (mut tx, owner, source) = setup(20_261_202);
    add_skill(&mut tx, owner, DCDESHI);
    // Heal the lord to full so the wounded precondition is false.
    let max = tx.state().players.get(owner).unwrap().max_hp;
    tx.state_mut().players.get_mut(owner).unwrap().hp = max as i32;
    let hp = max as i32;
    let max_hp_before = tx.state().players.get(owner).unwrap().max_hp;
    let hand_before = tx.state().hand_count(owner);
    let sha_hand_before = hand_sha_kind(&tx, owner).iter().filter(|k| **k == CardKind::Sha).count();

    let events = run_effects(
        &mut tx,
        vec![EffectRequest::DealCardDamage {
            source,
            target: owner,
            amount: 1,
            is_lose_hp: false,
            nature: damage_nature::NORMAL,
            card: CardId::NONE,
            card_kind: CardKind::Sha.tag(),
            card_flags: 0,
        }],
    )
    .unwrap();
    assert!(events.iter().any(|event| matches!(event, DomainEvent::DamageDealt { .. })));
    assert_eq!(tx.state().players.get(owner).unwrap().hp, hp - 1);
    assert_eq!(
        tx.state().players.get(owner).unwrap().max_hp,
        max_hp_before,
        "max HP only changes when the trigger fires"
    );
    assert_eq!(tx.state().hand_count(owner), hand_before);
    assert_eq!(
        hand_sha_kind(&tx, owner).iter().filter(|k| **k == CardKind::Sha).count(),
        sha_hand_before
    );
}

#[test]
fn wounded_owner_takes_non_sha_damage_unchanged() {
    let (mut tx, owner, source) = setup(20_261_203);
    add_skill(&mut tx, owner, DCDESHI);
    let hp = tx.state().players.get(owner).unwrap().hp;
    tx.state_mut().players.get_mut(owner).unwrap().hp = hp - 1;
    let max_hp_before = tx.state().players.get(owner).unwrap().max_hp;
    let hand_before = tx.state().hand_count(owner);

    let events = run_effects(
        &mut tx,
        vec![EffectRequest::DealCardDamage {
            source,
            target: owner,
            amount: 1,
            is_lose_hp: false,
            nature: damage_nature::FIRE,
            card: CardId::NONE,
            card_kind: CardKind::TrickHuogong.tag(),
            card_flags: 0,
        }],
    )
    .unwrap();
    assert!(events.iter().any(|event| matches!(event, DomainEvent::DamageDealt { .. })));
    assert_eq!(tx.state().players.get(owner).unwrap().hp, hp - 2);
    assert_eq!(tx.state().players.get(owner).unwrap().max_hp, max_hp_before);
    assert_eq!(tx.state().hand_count(owner), hand_before);
}
