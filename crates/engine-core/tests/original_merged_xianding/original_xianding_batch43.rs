//! Golden for XIANDING（限定包）批次四十三：
//! 奇绝（dcsbqijue）、毒逝·诅咒（dushi2）。

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

const DCSBQIJUE: u32 = 2_527_119_333;
const DUSHI2: u32 = 2_743_495_682;
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
fn package_exposes_batch43_skills_with_expected_contracts() {
    let package = package();
    assert!(package.manifest.rule_vm_abi_version >= 587);

    let qijue = package
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == DCSBQIJUE)
        .expect("dcsbqijue trigger");
    assert_eq!(qijue.event, "dying");
    assert!(qijue.optional);

    let cancel = package
        .modifier_index
        .iter()
        .find(|m| m.skill_def_id == DUSHI2 && m.target == "card_target_cancel")
        .expect("dushi2 tao cancel modifier");
    assert_eq!(cancel.selector.as_deref(), Some("tao"));
    assert_eq!(cancel.mode, "override");
    assert_eq!(cancel.value, 1);
}

#[test]
fn dcsbqijue_draws_two_and_recovers_two_for_dying_other() {
    let (mut tx, owner, first, _second) = setup(20_260_880);
    add_skill(&mut tx, owner, DCSBQIJUE);
    run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source: PlayerId::NONE,
            target: first,
            amount: 2,
            is_lose_hp: false,
            nature: 0,
        }],
    )
    .unwrap();
    let hp = tx.state().players.get(first).unwrap().hp;

    let dispatched = dispatch_for_events(
        &mut tx,
        &[DomainEvent::EnteredDying {
            player: first,
            source: PlayerId::NONE,
        }],
        &mut TriggerFireRecord::new(),
    );
    assert!(matches!(dispatched, Err(EffectError::ChoiceRequested)));
    let state = resolve(tx.state().clone(), owner, vec![1], false, 1);
    let state = resolve(state, owner, vec![first.raw()], false, 2);
    let state = resolve(state, owner, vec![2], false, 3);
    assert_eq!(state.players.get(first).unwrap().hp, hp + 2);
}

#[test]
fn dushi2_skill_can_be_attached() {
    let (mut tx, owner, _first, _second) = setup(20_260_881);
    add_skill(&mut tx, owner, DUSHI2);
    let has = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .any(|skill| skill.def == SkillDefId::new(DUSHI2));
    assert!(has);
}
