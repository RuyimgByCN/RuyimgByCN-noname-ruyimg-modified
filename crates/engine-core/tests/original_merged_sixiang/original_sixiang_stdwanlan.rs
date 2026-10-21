//! Golden coverage for 标贾逵「挽澜」rewritten from legacy JavaScript semantics.

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

const STD_JIAKUI: u32 = 3_406_075_592;
const STDZHONGZUO: u32 = 3_465_971_059;
const STDWANLAN: u32 = 3_664_427_038;
const GIFT_PROBE: u32 = 4_200_000_125;
const PACKAGE: &[u8] =
    include_bytes!("../../../../content/original/ported/sixiang/sixiang.npack.json");

fn package() -> content_schema::ContentPackage {
    content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap()
}

fn setup(with_probe: bool) -> (Transaction, PlayerId, PlayerId, Vec<CardId>) {
    let mut engine = Engine::new();
    engine.load_package(&package()).unwrap();
    if with_probe {
        let probe = content_compiler::compile_source_named(
            "stdwanlan-probe",
            r#"
skill gift_probe(4200000125) locked {
  trigger cards_moved owner_received_gift_from_other {
    effect lose_hp(self, 1);
  }
}
"#,
        )
        .unwrap();
        engine.load_package(&probe).unwrap();
    }
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(0),
            kind: CommandKind::StartGame {
                player_count: 2,
                rng_seed: 2_026_072_405,
                ruleset: "identity-standard".into(),
            },
        }),
        CommandResult::Accepted { .. }
    ));
    let owner = engine.state().turn.current;
    let target = engine
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .find(|player| *player != owner)
        .unwrap();
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
        skill_def: SkillDefId::new(STDWANLAN),
    });
    if with_probe {
        effects.push(EffectRequest::AddSkill {
            player: target,
            skill_def: SkillDefId::new(GIFT_PROBE),
        });
    }
    run_effects(&mut tx, effects).unwrap();
    let cards = vec![
        inject(&mut tx, owner, CardKind::Sha, Suit::Heart, 7),
        inject(&mut tx, owner, CardKind::Shan, Suit::Club, 8),
    ];
    {
        let participant = tx.state_mut().players.get_mut(target).unwrap();
        participant.hp = -1;
        participant.dying = true;
    }
    (tx, owner, target, cards)
}

fn inject(tx: &mut Transaction, owner: PlayerId, kind: CardKind, suit: Suit, rank: u8) -> CardId {
    let zone = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(94_000 + rank as u32),
        kind,
        suit,
        rank,
        weapon_range: 0,
        zone,
        owner,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn offer(tx: &mut Transaction, target: PlayerId) -> Result<Vec<EffectRequest>, EffectError> {
    dispatch_for_events(
        tx,
        &[DomainEvent::EnteredDying {
            player: target,
            source: PlayerId::NONE,
        }],
        &mut TriggerFireRecord::new(),
    )
}

fn resolve(
    tx: &Transaction,
    owner: PlayerId,
    accept: bool,
    key: u64,
) -> (GameState, Vec<DomainEvent>) {
    let (state, output) = apply_command(
        tx.state().clone(),
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(tx.state().revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id: ChoiceId::new(tx.state().resolution.pending_choice_id),
                selections: vec![u32::from(accept)],
                cancelled: false,
            },
        },
    );
    assert!(matches!(output.result, CommandResult::Accepted { .. }));
    (state, output.events)
}

fn wanlan_instance(state: &GameState, owner: PlayerId) -> engine_api::SkillId {
    state
        .skills
        .sorted_live()
        .into_iter()
        .find(|skill| skill.owner == owner && skill.def.raw() == STDWANLAN)
        .unwrap()
        .id
}

#[test]
fn package_publishes_abi125_skill_and_complete_std_jiakui() {
    let package = package();
    assert_eq!(package.manifest.rule_vm_abi_version, 562);
    assert_eq!(package.skill_index.len(), 181);
    assert_eq!(package.character_index.len(), 101);
    assert!(package.skill_index.iter().any(|skill| {
        skill.skill_def_id == STDWANLAN && skill.name == "stdwanlan" && !skill.locked
    }));
    assert!(package.character_index.iter().any(|character| {
        character.id == STD_JIAKUI
            && character.name_key == "std_jiakui"
            && character.max_hp == 3
            && character.skills == [STDZHONGZUO, STDWANLAN]
    }));
    let trigger = package
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == STDWANLAN)
        .unwrap();
    assert_eq!(
        (
            trigger.event.as_str(),
            trigger.filter.as_str(),
            trigger.optional
        ),
        (
            "dying",
            "owner_alive_other_dying_hand_nonempty_once_per_game",
            true
        )
    );
}

#[test]
fn accepting_gives_every_hand_card_then_recovers_after_nested_movement_trigger() {
    let (mut tx, owner, target, cards) = setup(true);
    assert_eq!(
        offer(&mut tx, target).unwrap_err(),
        EffectError::ChoiceRequested
    );
    let (saved, events) = resolve(&tx, owner, true, 1);
    assert_eq!(saved.hand_count(owner), 0);
    assert!(cards
        .iter()
        .all(|card| saved.cards.get(*card).unwrap().owner == target));
    assert_eq!(saved.players.get(target).unwrap().hp, 1);
    assert_eq!(
        saved
            .skills
            .get(wanlan_instance(&saved, owner))
            .unwrap()
            .lifetime_fire_count,
        1
    );
    let moved = events
        .iter()
        .position(|event| matches!(event, DomainEvent::CardsMoved { .. }))
        .unwrap();
    let nested_loss = events
        .iter()
        .position(|event| {
            matches!(
                event,
                DomainEvent::DamageDealt {
                    target: damaged,
                    amount: 1,
                    is_lose_hp: true,
                    ..
                } if *damaged == target
            )
        })
        .unwrap();
    let recovered = events
        .iter()
        .position(|event| {
            matches!(
                event,
                DomainEvent::HpRecovered {
                    player,
                    amount: 3,
                    ..
                } if *player == target
            )
        })
        .unwrap();
    assert!(moved < nested_loss && nested_loss < recovered);
}

#[test]
fn declining_does_not_consume_and_empty_self_or_spent_cases_do_not_offer() {
    let (mut tx, owner, target, _) = setup(false);
    let original_hand_count = tx.state().hand_count(owner);
    assert_eq!(
        offer(&mut tx, target).unwrap_err(),
        EffectError::ChoiceRequested
    );
    let (declined, _) = resolve(&tx, owner, false, 2);
    let skill = wanlan_instance(&declined, owner);
    assert_eq!(declined.skills.get(skill).unwrap().lifetime_fire_count, 0);
    assert_eq!(declined.hand_count(owner), original_hand_count);
    assert_eq!(declined.players.get(target).unwrap().hp, -1);

    let mut accepted_tx = Transaction::begin(declined);
    assert_eq!(
        offer(&mut accepted_tx, target).unwrap_err(),
        EffectError::ChoiceRequested
    );
    let (accepted, _) = resolve(&accepted_tx, owner, true, 3);
    let mut spent = Transaction::begin(accepted);
    let extra = inject(&mut spent, owner, CardKind::Tao, Suit::Heart, 9);
    spent.state_mut().players.get_mut(target).unwrap().hp = 0;
    spent.state_mut().players.get_mut(target).unwrap().dying = true;
    assert!(offer(&mut spent, target).unwrap().is_empty());
    assert!(!spent.state().has_pending_choice());
    assert_eq!(spent.state().cards.get(extra).unwrap().owner, owner);

    let (mut empty, empty_owner, empty_target, _) = setup(false);
    let cards = empty.state().hand_cards(empty_owner);
    let target_hand = empty
        .state()
        .zones
        .find(ZoneKind::Hand, empty_target)
        .unwrap();
    let owner_hand = empty
        .state()
        .zones
        .find(ZoneKind::Hand, empty_owner)
        .unwrap();
    run_effects(
        &mut empty,
        vec![EffectRequest::MoveCards {
            cards,
            to: target_hand,
            from: Some(owner_hand),
        }],
    )
    .unwrap();
    assert!(offer(&mut empty, empty_target).unwrap().is_empty());
    assert!(offer(&mut empty, empty_owner).unwrap().is_empty());
}
