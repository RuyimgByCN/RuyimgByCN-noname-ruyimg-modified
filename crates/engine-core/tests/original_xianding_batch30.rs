//! Golden for XIANDING（限定包）批次三十：
//! 醮影·摸牌（jiaoying3_draw）。

use engine_api::{
    ChoiceId, Command, CommandKind, CommandResult, DomainEvent, ExpectedRevision, IdempotencyKey,
    PlayerId, SkillDefId,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{ContinuationKind, GameState, Phase};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const JIAOYING3_DRAW: u32 = 4_012_724_406;
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/xianding/xianding.npack.json");

fn package() -> content_schema::ContentPackage {
    content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap()
}

fn setup(seed: u64) -> (Transaction, PlayerId, PlayerId, PlayerId) {
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package()).unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 3, seed, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let (owner, first, second) = (players[0], players[1], players[2]);
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
    (tx, owner, first, second)
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

fn resolve(
    state: GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    cancelled: bool,
    key: u64,
) -> GameState {
    let revision = state.revision;
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    let (state, result) = apply_command(
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
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    state
}

#[test]
fn package_exposes_batch30_skill_with_expected_contracts() {
    let package = package();
    assert!(package.manifest.rule_vm_abi_version >= 587);

    let trigger = package
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == JIAOYING3_DRAW)
        .expect("jiaoying3_draw trigger");
    assert_eq!(trigger.event, "phase_end");
    assert_eq!(trigger.filter, "owner_is_target");
    assert!(trigger.optional);
}

#[test]
fn jiaoying3_draw_draws_target_to_five_cards_at_phase_end() {
    let (mut tx, owner, first, _second) = setup(20_260_890);
    add_skill(&mut tx, owner, JIAOYING3_DRAW);
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::End;
    let before = tx.state().hand_count(first);

    let dispatched = dispatch_for_events(
        &mut tx,
        &[DomainEvent::TurnStateChanged {
            current: owner,
            phase: engine_api::TurnPhaseTag::End,
        }],
        &mut TriggerFireRecord::new(),
    );
    assert!(matches!(dispatched, Err(EffectError::ChoiceRequested)));
    assert_eq!(
        tx.state().resolution.continuation_kind,
        ContinuationKind::OptionalTrigger
    );
    let state = resolve(tx.state().clone(), owner, vec![1], false, 1);
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::ChooseTargetDraw
    );
    let state = resolve(state, owner, vec![first.raw()], false, 2);
    let after = state.hand_count(first);
    assert!(after > before);
    assert!(after <= 5, "hand should be capped at five, got {}", after);
}
