//! Golden coverage for BINGSHI 势陈群「法恩」「定品」.

use engine_api::{
    ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey, PlayerId,
    SkillDefId,
};
use engine_core::command::apply_command;
use engine_core::effect::{run_effects, EffectError};
use engine_core::state::{turn_flag, GameState, Phase};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::EffectRequest;

const POTFAEN: u32 = 4_194_455_770;
const POTDINGPIN: u32 = 2_518_587_448;
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/bingshi/bingshi.npack.json");

fn setup(skill_def: u32) -> (GameState, PlayerId, Vec<PlayerId>, engine_api::SkillId) {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(518),
            kind: CommandKind::StartGame {
                player_count: 3,
                rng_seed: 518,
                ruleset: "identity-standard".into(),
            },
        }),
        CommandResult::Accepted { .. }
    ));
    let owner = engine.state().turn.current;
    let others = engine
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .filter(|player| *player != owner)
        .collect::<Vec<_>>();
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
        skill_def: SkillDefId::new(skill_def),
    });
    run_effects(&mut tx, effects).unwrap();
    let skill = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .find(|skill| skill.owner == owner && skill.def.raw() == skill_def)
        .unwrap()
        .id;
    (tx.commit().state, owner, others, skill)
}

fn choose(state: GameState, selections: Vec<u32>, cancelled: bool, key: u64) -> GameState {
    let actor = PlayerId::new(state.resolution.pending_observer);
    let (state, output) = apply_command(
        state.clone(),
        Command {
            actor,
            expected_revision: ExpectedRevision(state.revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id: ChoiceId::new(state.resolution.pending_choice_id),
                selections,
                cancelled,
            },
        },
    );
    assert!(matches!(output.result, CommandResult::Accepted { .. }));
    state
}

#[test]
fn package_publishes_complete_pot_chenqun_at_abi518() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 677);
    assert_eq!(
        package
            .character_index
            .iter()
            .find(|character| character.name_key == "pot_chenqun")
            .unwrap()
            .skills,
        [POTFAEN, POTDINGPIN]
    );
}

#[test]
fn faen_remembers_previous_user_and_applies_the_opposite_to_the_next_user() {
    let (state, owner, others, skill) = setup(POTFAEN);
    let target = others[0];
    let next = others[1];
    let mut tx = Transaction::begin(state);
    run_effects(
        &mut tx,
        vec![EffectRequest::ResolvePotchenqun {
            player: owner,
            target: owner,
            stage: 0,
            skill,
        }],
    )
    .unwrap();
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::ResolvePotchenqun {
                player: owner,
                target,
                stage: 0,
                skill,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    );
    let before_draw = tx.state().hand_count(target);
    let state = choose(tx.commit().state, vec![1], false, 1);
    assert_eq!(state.hand_count(target), before_draw + 1);

    let before_discard = state.hand_count(next);
    let mut tx = Transaction::begin(state);
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::ResolvePotchenqun {
                player: owner,
                target: next,
                stage: 0,
                skill,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    );
    assert_eq!(PlayerId::new(tx.state().resolution.pending_observer), next);
    let card = tx.state().resolution.continuation_payload[6];
    let state = choose(tx.commit().state, vec![card], false, 2);
    assert_eq!(state.hand_count(next), before_discard - 1);

    let mut tx = Transaction::begin(state);
    run_effects(
        &mut tx,
        vec![EffectRequest::ResolvePotchenqun {
            player: owner,
            target: owner,
            stage: 0,
            skill,
        }],
    )
    .unwrap();
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::ResolvePotchenqun {
                player: owner,
                target,
                stage: 0,
                skill,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    );
    let state = choose(tx.commit().state, vec![2], false, 3);
    let discard = state.resolution.continuation_payload[5];
    let state = choose(state, vec![discard], false, 4);
    let before_draw = state.hand_count(next);
    let mut tx = Transaction::begin(state);
    run_effects(
        &mut tx,
        vec![EffectRequest::ResolvePotchenqun {
            player: owner,
            target: next,
            stage: 0,
            skill,
        }],
    )
    .unwrap();
    assert_eq!(tx.state().hand_count(next), before_draw + 1);
}

#[test]
fn dingpin_queues_a_real_draw_phase_for_a_maximum_loss_target() {
    let (state, owner, others, skill) = setup(POTDINGPIN);
    let target = others[0];
    let mut tx = Transaction::begin(state);
    let lost = tx.state().hand_cards(target)[0];
    tx.state_mut().turn.record_card_loss(target, lost);
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::End;
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::ResolvePotchenqun {
                player: owner,
                target: owner,
                stage: 1,
                skill,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    );
    let state = choose(tx.commit().state, vec![target.raw()], false, 3);
    assert_eq!(state.turn.extra_draw_phase_queue, [target]);
    let before = state.hand_count(target);
    let mut tx = Transaction::begin(state);
    run_effects(&mut tx, vec![EffectRequest::RemoveSkill { skill }]).unwrap();
    assert_eq!(tx.state().turn.extra_draw_phase_queue, [target]);
    tx.state_mut()
        .turn
        .set_flag(turn_flag::TURN_END_DISPATCHED, 1);
    run_effects(&mut tx, vec![EffectRequest::AdvanceTurn]).unwrap();
    assert_eq!(tx.state().turn.current, target);
    assert_eq!(tx.state().turn.phase, Phase::Draw);
    assert!(tx.state().turn.extra_draw_phase_active);
    assert_eq!(tx.state().hand_count(target), before + 2);
    let state = tx.commit().state;
    let (state, output) = apply_command(
        state.clone(),
        Command {
            actor: target,
            expected_revision: ExpectedRevision(state.revision),
            idempotency_key: IdempotencyKey(4),
            kind: CommandKind::EndPhase,
        },
    );
    assert!(matches!(output.result, CommandResult::Accepted { .. }));
    assert!(!state.turn.extra_draw_phase_active);
    assert_eq!(state.turn.phase, Phase::Prepare);
}
