//! Golden for XIANDING（限定包）批次三十三：
//! 识锦（dcsbshijin）、斗威（dcsbdouwei）。

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{ContinuationKind, GameState, Phase};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const DCSBSHIJIN: u32 = 3_506_294_264;
const DCSBDOUWEI: u32 = 2_217_274_823;
const PACKAGE: &[u8] =
    include_bytes!("../../../../content/original/ported/xianding/xianding.npack.json");

fn package() -> content_schema::ContentPackage {
    content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap()
}

fn setup(seed: u64) -> (Transaction, PlayerId, PlayerId, PlayerId) {
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content)
        .load_package(&package())
        .unwrap();
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
fn package_exposes_batch33_skills_with_expected_contracts() {
    let package = package();
    assert!(package.manifest.rule_vm_abi_version >= 587);

    let shijin = package
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == DCSBSHIJIN)
        .expect("dcsbshijin trigger");
    assert_eq!(shijin.event, "damage_before");
    assert_eq!(shijin.filter, "owner_is_target");
    assert!(!shijin.optional);

    let douwei = package
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == DCSBDOUWEI)
        .expect("dcsbdouwei trigger");
    assert_eq!(douwei.event, "dying");
    assert!(douwei.optional);
}

#[test]
fn dcsbshijin_prevents_damage_and_draws() {
    let (mut tx, owner, _first, _second) = setup(20_260_866);
    add_skill(&mut tx, owner, DCSBSHIJIN);
    let before = tx.state().hand_count(owner);
    let hp = tx.state().players.get(owner).unwrap().hp;

    let err = run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source: PlayerId::NONE,
            target: owner,
            amount: 2,
            is_lose_hp: false,
            nature: 0,
        }],
    );
    assert!(err.is_ok(), "damage should resolve without choice");
    assert_eq!(tx.state().players.get(owner).unwrap().hp, hp);
    assert_eq!(tx.state().hand_count(owner), before + 1);
}

#[test]
fn dcsbdouwei_recovers_when_other_enters_dying() {
    let (mut tx, owner, first, _second) = setup(20_260_867);
    add_skill(&mut tx, owner, DCSBDOUWEI);
    run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source: PlayerId::NONE,
            target: owner,
            amount: 1,
            is_lose_hp: false,
            nature: 0,
        }],
    )
    .unwrap();
    let hp = tx.state().players.get(owner).unwrap().hp;

    let dispatched = dispatch_for_events(
        &mut tx,
        &[DomainEvent::EnteredDying {
            player: first,
            source: PlayerId::NONE,
        }],
        &mut TriggerFireRecord::new(),
    );
    assert!(matches!(dispatched, Err(EffectError::ChoiceRequested)));
    assert_eq!(
        tx.state().resolution.continuation_kind,
        ContinuationKind::OptionalTrigger
    );
    let state = resolve(tx.state().clone(), owner, vec![1], false, 1);
    assert_eq!(state.players.get(owner).unwrap().hp, hp + 1);
}
