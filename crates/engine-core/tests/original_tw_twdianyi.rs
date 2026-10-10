//! Golden for TW 王粲「典仪」rewritten from legacy JavaScript semantics.

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

const DIANYI: u32 = 3_416_973_271;
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/tw/tw.npack.json");

fn isolated(hand: u32) -> (Transaction, PlayerId, PlayerId) {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(DIANYI as u64),
            kind: CommandKind::StartGame {
                player_count: 2,
                rng_seed: DIANYI as u64,
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
            skill_def: SkillDefId::new(DIANYI),
        },
    ]);
    run_effects(&mut tx, effects).unwrap();
    (tx, owner, other)
}

fn phase_end_effects(tx: &mut Transaction, player: PlayerId) -> Vec<EffectRequest> {
    dispatch_for_events(
        tx,
        &[DomainEvent::TurnStateChanged {
            current: player,
            phase: TurnPhaseTag::End,
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap()
}

#[test]
fn committed_package_exposes_both_mutually_exclusive_turn_damage_contracts() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 733);
    let filters: Vec<_> = package
        .trigger_index
        .iter()
        .filter(|trigger| trigger.skill_def_id == DIANYI)
        .map(|trigger| trigger.filter.as_str())
        .collect();
    assert_eq!(
        filters,
        vec![
            "owner_is_target_turn_damage",
            "owner_is_target_no_turn_damage"
        ]
    );
}

#[test]
fn damage_branch_discards_every_hand_card_and_other_turn_end_does_nothing() {
    let (mut tx, owner, other) = isolated(5);
    assert!(phase_end_effects(&mut tx, other).is_empty());
    tx.state_mut()
        .turn
        .set_flag(engine_core::state::turn_flag::DAMAGE_DEALT, 1);
    let effects = phase_end_effects(&mut tx, owner);
    run_effects(&mut tx, effects).unwrap();
    assert_eq!(tx.state().hand_count(owner), 0);
}

#[test]
fn no_damage_branch_draws_up_to_four() {
    let (mut tx, owner, _) = isolated(2);
    let effects = phase_end_effects(&mut tx, owner);
    run_effects(&mut tx, effects).unwrap();
    assert_eq!(tx.state().hand_count(owner), 4);
    assert!(!tx.state().has_pending_choice());
}

#[test]
fn no_damage_branch_forces_exact_excess_selection() {
    let (mut tx, owner, _) = isolated(6);
    let original = tx.state().hand_cards(owner);
    let effects = phase_end_effects(&mut tx, owner);
    assert_eq!(
        run_effects(&mut tx, effects),
        Err(EffectError::ChoiceRequested)
    );
    assert_eq!(
        tx.state().resolution.continuation_kind,
        ContinuationKind::AdjustHandTo
    );
    let discarded = vec![original[1].raw(), original[4].raw()];
    let (state, result) = apply_command(
        tx.state().clone(),
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(tx.state().revision),
            idempotency_key: IdempotencyKey(DIANYI as u64 + 1),
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
