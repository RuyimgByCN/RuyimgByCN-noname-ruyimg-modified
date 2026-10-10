//! Golden for XIANDING（限定包）批次三十七：
//! 毒露（dcdulu）、猝袭（dccuxi）。

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

const DCDULU: u32 = 2_779_276_477;
const DCCUXI: u32 = 4_093_257_724;
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
fn package_exposes_batch37_skills_with_expected_contracts() {
    let package = package();
    assert!(package.manifest.rule_vm_abi_version >= 587);

    let dulu = package
        .active_index
        .iter()
        .find(|active| active.skill_def_id == DCDULU)
        .expect("dcdulu active");
    assert_eq!(dulu.window, "play_phase");
    assert_eq!(dulu.uses_per_turn, Some(1));
    assert!(dulu
        .targets
        .as_ref()
        .is_some_and(|t| t.min == 1 && t.max == 3 && !t.allow_self));

    let cuxi = package
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == DCCUXI)
        .expect("dccuxi trigger");
    assert_eq!(cuxi.event, "phase_begin");
    assert_eq!(cuxi.filter, "owner_alive_not_target");
    assert!(cuxi.optional);
}

#[test]
fn dcdulu_damages_targets_on_activation() {
    let (mut tx, owner, first, _second) = setup(20_260_872);
    add_skill(&mut tx, owner, DCDULU);
    let hp = tx.state().players.get(first).unwrap().hp;
    let skill_id = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .find(|skill| skill.def == SkillDefId::new(DCDULU))
        .expect("live dcdulu")
        .id;
    let (state, result) = apply_command(
        tx.state().clone(),
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(tx.state().revision),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::ActivateSkill {
                skill: skill_id,
                targets: vec![first],
                card_targets: vec![],
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(first).unwrap().hp, hp - 1);
}

#[test]
fn dccuxi_discards_two_and_damages_at_other_phase_begin() {
    let (mut tx, owner, first, _second) = setup(20_260_873);
    add_skill(&mut tx, owner, DCCUXI);
    tx.state_mut().turn.current = first;
    tx.state_mut().turn.phase = Phase::Prepare;
    let hp = tx.state().players.get(first).unwrap().hp;

    let dispatched = dispatch_for_events(
        &mut tx,
        &[DomainEvent::TurnStateChanged {
            current: first,
            phase: engine_api::TurnPhaseTag::Prepare,
        }],
        &mut TriggerFireRecord::new(),
    );
    assert!(matches!(dispatched, Err(EffectError::ChoiceRequested)));
    assert_eq!(
        tx.state().resolution.continuation_kind,
        ContinuationKind::OptionalTrigger
    );
    let state = resolve(tx.state().clone(), owner, vec![1], false, 1);
    assert_eq!(state.players.get(first).unwrap().hp, hp - 1);
}
