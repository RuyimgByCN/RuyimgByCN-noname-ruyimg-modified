//! Golden tests for COLLAB「反转」rewritten from the legacy JavaScript semantics.

use engine_api::{
    ChoiceId, Command, CommandKind, CommandResult, DomainEvent, ExpectedRevision, IdempotencyKey,
    PlayerId, SkillDefId, TurnPhaseTag,
};
use engine_core::command::apply_command;
use engine_core::effect::{run_effects, EffectError};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use engine_core::Engine;
use rule_api::EffectRequest;

const DCFANZHUAN: u32 = 2_353_467_697;
const DCNIYUN: u32 = 3_024_539_645;
const CHARACTER: u32 = 2_549_428_417;
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/collab/collab.npack.json");

fn started_tx(seed: u64) -> (Transaction, PlayerId) {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(seed),
        kind: CommandKind::StartGame {
            player_count: 2,
            rng_seed: seed,
            ruleset: "identity-standard".into(),
        },
    });
    engine.load_package(&package).unwrap();
    let owner = engine.state().turn.current;
    let mut tx = Transaction::begin(engine.state().clone());
    // The standard roster includes choice-bearing characters (e.g. 夏侯惇 刚烈)
    // whose booted presence makes these tests roster-dependent; this test only
    // exercises DCFANZHUAN, so drop every booted skill.
    let booted_skills: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    if !booted_skills.is_empty() {
        run_effects(&mut tx, booted_skills).unwrap();
    }
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(DCFANZHUAN),
        }],
    )
    .unwrap();
    (tx, owner)
}

fn begin_turn(tx: &mut Transaction, current: PlayerId) -> Result<(), EffectError> {
    dispatch_for_events(
        tx,
        &[DomainEvent::TurnStateChanged {
            current,
            phase: TurnPhaseTag::Prepare,
        }],
        &mut TriggerFireRecord::new(),
    )
    .map(|_| ())
}

fn resolve(tx: Transaction, accept: bool, key: u64) -> Transaction {
    let state = tx.state().clone();
    let observer = PlayerId::new(state.resolution.pending_observer);
    let (state, pipe) = apply_command(
        state,
        Command {
            actor: observer,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id: ChoiceId::new(tx.state().resolution.pending_choice_id),
                selections: vec![u32::from(accept)],
                cancelled: false,
            },
        },
    );
    assert!(matches!(pipe.result, CommandResult::Accepted { .. }));
    Transaction::begin(state)
}

#[test]
fn package_contains_complete_initially_damaged_character() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == CHARACTER)
        .expect("逆转之神 character entry");
    assert_eq!((character.initial_hp, character.max_hp), (4, 5));
    assert_eq!(character.skills, vec![DCFANZHUAN, DCNIYUN]);
}

#[test]
fn adjusts_both_directions_draws_delta_and_fires_once_per_round() {
    let (mut tx, owner) = started_tx(2_026_071_510);
    let other = tx
        .state()
        .players
        .iter_live()
        .find(|player| player.id != owner)
        .unwrap()
        .id;
    {
        let player = tx.state_mut().players.get_mut(owner).unwrap();
        player.hp = 1;
        player.max_hp = 5;
    }
    let hand = tx.state().hand_count(owner);
    assert_eq!(
        begin_turn(&mut tx, other),
        Err(EffectError::ChoiceRequested)
    );
    let mut tx = resolve(tx, true, 1);
    assert_eq!(tx.state().players.get(owner).unwrap().hp, 4);
    assert_eq!(tx.state().hand_count(owner), hand + 3);

    assert_eq!(begin_turn(&mut tx, owner), Ok(()));
    assert!(!tx.state().has_pending_choice());

    tx.state_mut().turn.round += 1;
    let hand = tx.state().hand_count(owner);
    assert_eq!(
        begin_turn(&mut tx, other),
        Err(EffectError::ChoiceRequested)
    );
    let tx = resolve(tx, true, 2);
    assert_eq!(tx.state().players.get(owner).unwrap().hp, 1);
    assert_eq!(tx.state().hand_count(owner), hand + 3);
}

#[test]
fn declining_does_not_consume_the_round_limit() {
    let (mut tx, owner) = started_tx(2_026_071_511);
    assert_eq!(
        begin_turn(&mut tx, owner),
        Err(EffectError::ChoiceRequested)
    );
    let mut tx = resolve(tx, false, 1);
    assert_eq!(
        begin_turn(&mut tx, owner),
        Err(EffectError::ChoiceRequested)
    );
}
