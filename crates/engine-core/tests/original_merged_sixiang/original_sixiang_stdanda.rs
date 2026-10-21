//! Golden coverage for 标吴珂「谙达」rewritten from legacy JavaScript semantics.

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

const STD_WUKE: u32 = 4_182_138_442;
const STDANDA: u32 = 3_646_320_211;
const STDZHUGUO: u32 = 3_147_233_152;
const PACKAGE: &[u8] =
    include_bytes!("../../../../content/original/ported/sixiang/sixiang.npack.json");

fn package() -> content_schema::ContentPackage {
    content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap()
}

fn setup(with_source_card: bool) -> (Transaction, PlayerId, PlayerId, PlayerId, Option<CardId>) {
    let mut engine = Engine::new();
    engine.load_package(&package()).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(0),
            kind: CommandKind::StartGame {
                player_count: 3,
                rng_seed: 2_026_072_406,
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
        skill_def: SkillDefId::new(STDANDA),
    });
    run_effects(&mut tx, effects).unwrap();
    clear_hand_and_equip(&mut tx, source);
    let card = with_source_card.then(|| inject(&mut tx, source));
    {
        let participant = tx.state_mut().players.get_mut(target).unwrap();
        participant.hp = 0;
        participant.dying = true;
    }
    (tx, owner, source, target, card)
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

fn inject(tx: &mut Transaction, owner: PlayerId) -> CardId {
    let zone = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(129_001),
        kind: CardKind::Sha,
        suit: Suit::Heart,
        rank: 7,
        weapon_range: 0,
        zone,
        owner,
        face_up: false,
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

fn stdanda_instance(state: &GameState, owner: PlayerId) -> engine_api::SkillId {
    state
        .skills
        .sorted_live()
        .into_iter()
        .find(|skill| skill.owner == owner && skill.def.raw() == STDANDA)
        .unwrap()
        .id
}

#[test]
fn package_publishes_abi131_skill_and_complete_std_wuke() {
    let package = package();
    assert_eq!(package.manifest.rule_vm_abi_version, 562);
    assert_eq!(package.skill_index.len(), 181);
    assert_eq!(package.character_index.len(), 101);
    assert!(package
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == STDANDA && skill.name == "stdanda" && !skill.locked));
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == STD_WUKE)
        .unwrap();
    assert_eq!(character.name_key, "std_wuke");
    assert_eq!(character.sex, "female");
    assert_eq!(character.group, "wu");
    assert_eq!(character.max_hp, 3);
    assert_eq!(character.skills, [STDANDA, STDZHUGUO]);
    let trigger = package
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == STDANDA)
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
fn accepted_source_gift_is_observed_by_source_and_does_not_recover() {
    let (mut tx, owner, source, target, card) = setup(true);
    assert_eq!(
        offer(&mut tx, source, target).unwrap_err(),
        EffectError::ChoiceRequested
    );
    let (offered, _) = resolve(tx.state().clone(), owner, vec![1], false, 1);
    assert!(offered.has_pending_choice());
    assert_eq!(offered.resolution.pending_observer, source.raw());
    let card = card.unwrap();
    let (given, events) = resolve(offered, source, vec![card.raw()], false, 2);
    assert_eq!(given.cards.get(card).unwrap().owner, target);
    assert_eq!(given.players.get(target).unwrap().hp, 0);
    assert!(given.players.get(target).unwrap().dying);
    assert!(events
        .iter()
        .any(|event| matches!(event, DomainEvent::CardsMoved { .. })));
    assert!(!events
        .iter()
        .any(|event| matches!(event, DomainEvent::HpRecovered { .. })));
    let skill = given.skills.get(stdanda_instance(&given, owner)).unwrap();
    assert_eq!(skill.round_fire_round, given.turn.round);

    let mut spent = Transaction::begin(given);
    assert!(offer(&mut spent, source, target).unwrap().is_empty());
    assert!(!spent.state().has_pending_choice());
}

#[test]
fn source_decline_recovers_dying_target_by_one() {
    let (mut tx, owner, source, target, _) = setup(true);
    assert_eq!(
        offer(&mut tx, source, target).unwrap_err(),
        EffectError::ChoiceRequested
    );
    let (offered, _) = resolve(tx.state().clone(), owner, vec![1], false, 3);
    let (recovered, events) = resolve(offered, source, Vec::new(), true, 4);
    assert_eq!(recovered.players.get(target).unwrap().hp, 1);
    assert!(!recovered.players.get(target).unwrap().dying);
    assert!(events.iter().any(|event| {
        matches!(
            event,
            DomainEvent::HpRecovered {
                player,
                amount: 1,
                ..
            } if *player == target
        )
    }));
}

#[test]
fn source_without_cards_recovers_immediately_after_owner_accepts() {
    let (mut tx, owner, source, target, card) = setup(false);
    assert!(card.is_none());
    assert_eq!(
        offer(&mut tx, source, target).unwrap_err(),
        EffectError::ChoiceRequested
    );
    let (recovered, events) = resolve(tx.state().clone(), owner, vec![1], false, 5);
    assert!(!recovered.has_pending_choice());
    assert_eq!(recovered.players.get(target).unwrap().hp, 1);
    assert!(events
        .iter()
        .any(|event| matches!(event, DomainEvent::HpRecovered { player, amount: 1, .. } if *player == target)));
}

#[test]
fn missing_living_source_or_non_dying_target_does_not_offer() {
    let (mut tx, _, source, target, _) = setup(true);
    assert!(offer(&mut tx, PlayerId::NONE, target).unwrap().is_empty());
    tx.state_mut().players.get_mut(target).unwrap().hp = 1;
    tx.state_mut().players.get_mut(target).unwrap().dying = false;
    assert!(offer(&mut tx, source, target).unwrap().is_empty());
    assert!(!tx.state().has_pending_choice());
}
