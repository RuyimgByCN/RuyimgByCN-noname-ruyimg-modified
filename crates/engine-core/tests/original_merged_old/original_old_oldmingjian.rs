//! Golden coverage for old「明鉴」rewritten from legacy JavaScript semantics.

use engine_api::{
    ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey, PlayerId,
    SkillDefId,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{GameState, Phase};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const OLDMINGJIAN: u32 = 2_370_380_501;
const SOURCE: &str = include_str!("../../../../content/original/ported/old/oldmingjian.skill");
const PACKAGE: &[u8] = include_bytes!("../../../../content/original/ported/old/old.npack.json");

fn submit(state: GameState, actor: PlayerId, kind: CommandKind, key: u64) -> GameState {
    let revision = state.revision;
    let (state, result) = apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(key),
            kind,
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    state
}

#[test]
fn gives_all_hand_then_target_gets_only_play_phase_even_turned_over() {
    let package = content_compiler::compile_source_named("oldmingjian-test", SOURCE).unwrap();
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content)
        .load_package(&package)
        .unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 3, 2_026_072_809, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let owner = players[0];
    let successor = players[1];
    let target = players[2];
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(OLDMINGJIAN),
    });
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    tx.state_mut().players.get_mut(target).unwrap().turned_over = true;
    let owner_hand = tx.state().hand_count(owner);
    let target_hand = tx.state().hand_count(target);
    let skill = tx.state().players.get(owner).unwrap().skills[0];
    assert!(matches!(
        run_effects(
            &mut tx,
            vec![EffectRequest::ResolveOldMingjian {
                player: owner,
                target: PlayerId::NONE,
                stage: 0,
                skill,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    ));
    let state = tx.commit().state;
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    let state = submit(
        state,
        owner,
        CommandKind::ResolveChoice {
            choice_id,
            selections: vec![target.raw()],
            cancelled: false,
        },
        1,
    );

    assert_eq!(state.hand_count(owner), 0);
    assert_eq!(state.hand_count(target), target_hand + owner_hand);
    assert_eq!(state.turn.current, target);
    assert_eq!(state.turn.phase, Phase::Play);
    assert!(state.turn.extra_play_phase_active);
    assert!(state.players.get(target).unwrap().turned_over);

    let state = submit(state, target, CommandKind::EndPhase, 2);
    assert_eq!(state.turn.current, successor);
    assert_eq!(state.turn.phase, Phase::Prepare);
    assert!(!state.turn.extra_play_phase_active);
    assert!(state.players.get(target).unwrap().turned_over);
}

#[test]
fn package_publishes_oldmingjian_without_javascript() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 275);
    assert!(package
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == OLDMINGJIAN && skill.name == "oldmingjian"));
    assert!(!serde_json::to_string(&package).unwrap().contains(".js"));
}
