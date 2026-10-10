//! Golden for SP2 严夫人「匿伏」rewritten from legacy JavaScript semantics.

use engine_api::{
    ChoiceId, Command, CommandKind, CommandResult, DomainEvent, ExpectedRevision, IdempotencyKey,
    PlayerId, SkillDefId, TurnPhaseTag,
};
use engine_core::command::apply_command;
use engine_core::effect::{run_effects, EffectError};
use engine_core::state::ContinuationKind;
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use engine_core::Engine;
use rule_api::EffectRequest;

const NIFU: u32 = 2_611_616_498;
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/sp2/sp2.npack.json");

fn isolated(hand: u32) -> (Transaction, PlayerId, PlayerId) {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(NIFU as u64),
            kind: CommandKind::StartGame {
                player_count: 2,
                rng_seed: NIFU as u64,
                ruleset: "identity-standard".into(),
            },
        }),
        CommandResult::Accepted { .. }
    ));
    let owner = engine.state().turn.current;
    let other = engine
        .state()
        .players
        .iter_live()
        .find(|player| player.id != owner)
        .unwrap()
        .id;
    let mut tx = Transaction::begin(engine.state().clone());
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.extend([
        EffectRequest::DiscardFromHand {
            player: owner,
            count: u32::MAX,
        },
        EffectRequest::DrawCards {
            player: owner,
            count: hand,
        },
        EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(NIFU),
        },
    ]);
    run_effects(&mut tx, effects).unwrap();
    (tx, owner, other)
}

fn fire(tx: &mut Transaction, current: PlayerId) -> Result<(), EffectError> {
    let effects = dispatch_for_events(
        tx,
        &[DomainEvent::TurnStateChanged {
            current,
            phase: TurnPhaseTag::End,
        }],
        &mut TriggerFireRecord::new(),
    )?;
    run_effects(tx, effects).map(|_| ())
}

#[test]
fn committed_package_exposes_locked_global_phase_end_contract() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let trigger = package
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == NIFU)
        .unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 753);
    assert_eq!(
        (
            trigger.event.as_str(),
            trigger.filter.as_str(),
            trigger.optional
        ),
        ("phase_end", "owner_alive", false)
    );
}

#[test]
fn any_players_turn_end_draws_owner_up_to_four() {
    let (mut tx, owner, other) = isolated(2);
    fire(&mut tx, other).unwrap();
    assert_eq!(tx.state().hand_count(owner), 4);
}

#[test]
fn excess_hand_requires_owner_to_choose_exact_cards() {
    let (mut tx, owner, other) = isolated(6);
    let original = tx.state().hand_cards(owner);
    assert_eq!(fire(&mut tx, other), Err(EffectError::ChoiceRequested));
    assert_eq!(
        tx.state().resolution.continuation_kind,
        ContinuationKind::AdjustHandTo
    );
    let discarded = vec![original[0].raw(), original[5].raw()];
    let (state, result) = apply_command(
        tx.state().clone(),
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(tx.state().revision),
            idempotency_key: IdempotencyKey(NIFU as u64 + 1),
            kind: CommandKind::ResolveChoice {
                choice_id: ChoiceId::new(tx.state().resolution.pending_choice_id),
                selections: discarded.clone(),
                cancelled: false,
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    let hand = state.hand_cards(owner);
    assert_eq!(hand.len(), 4);
    assert!(discarded
        .into_iter()
        .all(|card| !hand.iter().any(|remaining| remaining.raw() == card)));
}
