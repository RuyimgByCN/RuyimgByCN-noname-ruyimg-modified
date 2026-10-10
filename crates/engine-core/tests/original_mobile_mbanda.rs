//! Golden coverage for mobile 吴珂「谙达」rewritten from legacy JavaScript semantics.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use engine_core::Engine;
use rule_api::EffectRequest;

const MBANDA: u32 = 2_849_838_127;
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/mobile/mobile.npack.json");

fn package() -> content_schema::ContentPackage {
    content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap()
}

fn setup(cards: &[(Suit, ZoneKind)]) -> (Transaction, PlayerId, PlayerId, PlayerId, Vec<CardId>) {
    let mut engine = Engine::new();
    engine.load_package(&package()).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(0),
            kind: CommandKind::StartGame {
                player_count: 3,
                rng_seed: 2_026_073_101,
                ruleset: "identity-standard".into(),
            },
        }),
        CommandResult::Accepted { .. }
    ));
    let owner = engine.state().turn.current;
    let others: Vec<_> = engine
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .filter(|player| *player != owner)
        .collect();
    let source = others[0];
    let target = others[1];
    let mut tx = Transaction::begin(engine.state().clone());
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(MBANDA),
    });
    run_effects(&mut tx, effects).unwrap();
    clear_hand_and_equip(&mut tx, source);
    let cards = cards
        .iter()
        .enumerate()
        .map(|(index, (suit, zone))| inject(&mut tx, source, *suit, *zone, index as u32))
        .collect();
    {
        let participant = tx.state_mut().players.get_mut(target).unwrap();
        participant.hp = 0;
        participant.dying = true;
    }
    (tx, owner, source, target, cards)
}

fn clear_hand_and_equip(tx: &mut Transaction, player: PlayerId) {
    let mut cards = tx.state().hand_cards(player);
    if let Some(equip) = tx.state().zones.find(ZoneKind::Equip, player) {
        cards.extend(tx.state().zones.get(equip).unwrap().cards.iter().copied());
    }
    if !cards.is_empty() {
        run_effects(tx, vec![EffectRequest::DiscardCards { player, cards }]).unwrap();
    }
}

fn inject(
    tx: &mut Transaction,
    owner: PlayerId,
    suit: Suit,
    zone_kind: ZoneKind,
    offset: u32,
) -> CardId {
    let zone = tx.state().zones.find(zone_kind, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(330_001 + offset),
        kind: CardKind::Sha,
        suit,
        rank: 7,
        weapon_range: 0,
        zone,
        owner,
        face_up: zone_kind == ZoneKind::Equip,
    });
    tx.state_mut().zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn offer(
    tx: &mut Transaction,
    source: PlayerId,
    target: PlayerId,
) -> Result<Vec<EffectRequest>, EffectError> {
    dispatch_for_events(
        tx,
        &[DomainEvent::EnteredDying {
            player: target,
            source,
        }],
        &mut TriggerFireRecord::new(),
    )
}

fn resolve(
    state: GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    cancelled: bool,
    key: u64,
) -> (GameState, Vec<DomainEvent>) {
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    let revision = state.revision;
    let (state, output) = apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections,
                cancelled,
            },
        },
    );
    assert!(matches!(output.result, CommandResult::Accepted { .. }));
    (state, output.events)
}

fn mbanda_instance(state: &GameState, owner: PlayerId) -> engine_api::SkillId {
    state
        .skills
        .sorted_live()
        .into_iter()
        .find(|skill| skill.owner == owner && skill.def.raw() == MBANDA)
        .unwrap()
        .id
}

#[test]
fn package_publishes_abi330_mbanda_trigger() {
    let package = package();
    assert_eq!(package.manifest.rule_vm_abi_version, 734);
    assert_eq!(package.skill_index.len(), 440);
    assert!(package
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == MBANDA && skill.name == "mbanda" && !skill.locked));
    let trigger = package
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == MBANDA)
        .unwrap();
    assert_eq!(
        (
            trigger.event.as_str(),
            trigger.filter.as_str(),
            trigger.optional
        ),
        (
            "dying",
            "owner_alive_source_alive_event_player_dying_once_per_round",
            true
        )
    );
}

#[test]
fn source_gives_different_colors_across_hand_and_equip_without_recovery() {
    let (mut tx, owner, source, target, cards) = setup(&[
        (Suit::Heart, ZoneKind::Hand),
        (Suit::Spade, ZoneKind::Equip),
    ]);
    assert_eq!(
        offer(&mut tx, source, target).unwrap_err(),
        EffectError::ChoiceRequested
    );
    let (offered, _) = resolve(tx.state().clone(), owner, vec![1], false, 1);
    assert_eq!(offered.resolution.pending_observer, source.raw());
    let (given, events) = resolve(
        offered,
        source,
        cards.iter().map(|card| card.raw()).collect(),
        false,
        2,
    );
    assert!(cards
        .iter()
        .all(|card| given.cards.get(*card).unwrap().owner == target));
    assert_eq!(given.players.get(target).unwrap().hp, 0);
    assert!(!events
        .iter()
        .any(|event| matches!(event, DomainEvent::HpRecovered { .. })));
    let skill = given.skills.get(mbanda_instance(&given, owner)).unwrap();
    assert_eq!(skill.round_fire_round, given.turn.round);

    let mut spent = Transaction::begin(given);
    assert!(offer(&mut spent, source, target).unwrap().is_empty());
}

#[test]
fn no_different_color_pair_recovers_immediately_after_activation() {
    let (mut tx, owner, source, target, _) = setup(&[
        (Suit::Heart, ZoneKind::Hand),
        (Suit::Diamond, ZoneKind::Equip),
    ]);
    assert_eq!(
        offer(&mut tx, source, target).unwrap_err(),
        EffectError::ChoiceRequested
    );
    let (recovered, events) = resolve(tx.state().clone(), owner, vec![1], false, 3);
    assert!(!recovered.has_pending_choice());
    assert_eq!(recovered.players.get(target).unwrap().hp, 1);
    assert!(events.iter().any(
        |event| matches!(event, DomainEvent::HpRecovered { player, amount: 1, .. } if *player == target)
    ));
}

#[test]
fn source_decline_recovers_and_same_color_pair_is_rejected() {
    let (mut tx, owner, source, target, cards) = setup(&[
        (Suit::Heart, ZoneKind::Hand),
        (Suit::Diamond, ZoneKind::Hand),
        (Suit::Spade, ZoneKind::Equip),
    ]);
    assert_eq!(
        offer(&mut tx, source, target).unwrap_err(),
        EffectError::ChoiceRequested
    );
    let (offered, _) = resolve(tx.state().clone(), owner, vec![1], false, 4);
    let choice_id = ChoiceId::new(offered.resolution.pending_choice_id);
    let revision = offered.revision;
    let (unchanged, output) = apply_command(
        offered,
        Command {
            actor: source,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(5),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections: vec![cards[0].raw(), cards[1].raw()],
                cancelled: false,
            },
        },
    );
    assert!(matches!(output.result, CommandResult::Rejected { .. }));
    assert!(unchanged.has_pending_choice());

    let (recovered, _) = resolve(unchanged, source, Vec::new(), true, 6);
    assert_eq!(recovered.players.get(target).unwrap().hp, 1);
}
