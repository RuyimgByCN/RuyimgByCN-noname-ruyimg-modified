//! Golden coverage for SP 沙摩柯「蒺藜」.

use engine_api::{
    ChoiceId, Command, CommandKind, CommandResult, DomainEvent, ExpectedRevision, IdempotencyKey,
    PlayerId, SkillDefId,
};
use engine_core::command::apply_command;
use engine_core::effect::{run_effects, EffectError};
use engine_core::query::{query, QueryContext};
use engine_core::state::{CardKind, Phase};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord, BOOTSTRAP_QICAO};
use engine_core::Engine;
use rule_api::{EffectRequest, QueryRequest, QueryResponse};

const SHAMOKE: u32 = 3_007_668_710;
const GZJILI: u32 = 3_565_936_884;
const PACKAGE: &[u8] = include_bytes!("../../../../content/original/ported/sp/sp.npack.json");

fn setup(extra_range: bool) -> (Transaction, PlayerId, engine_api::CardId) {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let shamoke = package
        .character_index
        .iter()
        .find(|character| character.id == SHAMOKE)
        .unwrap();
    assert_eq!(shamoke.skills, vec![GZJILI]);

    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    engine.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(0),
        kind: CommandKind::StartGame {
            player_count: 2,
            rng_seed: 20_260_715,
            ruleset: "identity-standard".into(),
        },
    });
    let owner = engine.state().turn.current;
    let card = engine.state().hand_cards(owner)[0];
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
        skill_def: SkillDefId::new(GZJILI),
    });
    if extra_range {
        effects.push(EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(BOOTSTRAP_QICAO),
        });
    }
    run_effects(&mut tx, effects).unwrap();
    (tx, owner, card)
}

fn action_count(tx: &Transaction, player: PlayerId) -> i32 {
    match query(
        tx.state(),
        &QueryRequest::CardActionCount { player },
        &mut QueryContext::new(),
    ) {
        QueryResponse::Int(count) => count,
        other => panic!("unexpected query result: {other:?}"),
    }
}

fn accept(tx: &Transaction, owner: PlayerId) -> engine_core::state::GameState {
    let (state, result) = apply_command(
        tx.state().clone(),
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::ResolveChoice {
                choice_id: ChoiceId::new(tx.state().resolution.pending_choice_id),
                selections: vec![1],
                cancelled: false,
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    state
}

#[test]
fn first_card_use_at_default_range_offers_draw_one() {
    let (mut tx, owner, card) = setup(false);
    let hand = tx.state().hand_count(owner);
    let paused = dispatch_for_events(
        &mut tx,
        &[DomainEvent::CardUsed {
            player: owner,
            card,
            targets: vec![],
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap_err();
    assert_eq!(paused, EffectError::ChoiceRequested);
    assert_eq!(action_count(&tx, owner), 1);

    let state = accept(&tx, owner);
    assert_eq!(state.hand_count(owner), hand + 1);
}

#[test]
fn use_plus_response_reaches_range_two_and_count_resets_next_turn() {
    let (mut tx, owner, card) = setup(true);
    let hand = tx.state().hand_count(owner);
    let effects = dispatch_for_events(
        &mut tx,
        &[DomainEvent::CardUsed {
            player: owner,
            card,
            targets: vec![],
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    assert!(effects.is_empty());
    assert_eq!(action_count(&tx, owner), 1);

    let paused = dispatch_for_events(
        &mut tx,
        &[DomainEvent::CardResponded {
            player: owner,
            respond_to: PlayerId::NONE,
            card: engine_api::CardId::NONE,
            response_kind: CardKind::Shan.tag(),
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap_err();
    assert_eq!(paused, EffectError::ChoiceRequested);
    assert_eq!(action_count(&tx, owner), 2);

    let state = accept(&tx, owner);
    assert_eq!(state.hand_count(owner), hand + 2);
    let mut next_turn = Transaction::begin(state);
    run_effects(
        &mut next_turn,
        vec![
            EffectRequest::SetTurnState {
                current: owner,
                phase: Phase::End.to_u8(),
            },
            EffectRequest::SetTurnState {
                current: owner,
                phase: Phase::Prepare.to_u8(),
            },
        ],
    )
    .unwrap();
    assert_eq!(action_count(&next_turn, owner), 0);
}
