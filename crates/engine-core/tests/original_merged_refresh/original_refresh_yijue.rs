//! Golden coverage for 关羽「义绝」.

use engine_api::{
    CardId, ChoiceId, Command, CommandKind, CommandResult, DomainEvent, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{GameState, Phase};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const YIJUE: u32 = 3_159_500_535;
const YIJUE_WIN: u32 = 3_572_578_464;
const YIJUE_LOSE: u32 = 3_300_215_091;
const YIJUE2: u32 = 2_542_175_146;
const SOURCE: &str = concat!(
    include_str!("../../../../content/original/ported/refresh/yijue.skill"),
    include_str!("../../../../content/original/ported/refresh/yijue2.skill"),
);

fn setup() -> (GameState, PlayerId, PlayerId) {
    let package = content_compiler::compile_source_named("yijue-test", SOURCE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 2, 20_260_801, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content)
        .load_package(&package)
        .unwrap();
    let players = tx.state().players.alive_by_seat();
    let effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.current = players[0];
    tx.state_mut().turn.phase = Phase::Play;
    (tx.commit().state, players[0], players[1])
}

fn add_skill(state: GameState, player: PlayerId, def: u32) -> GameState {
    let mut tx = Transaction::begin(state);
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player,
            skill_def: SkillDefId::new(def),
        }],
    )
    .unwrap();
    tx.commit().state
}

fn compare_effects(
    tx: &mut Transaction,
    owner: PlayerId,
    target: PlayerId,
    winner: PlayerId,
) -> Vec<EffectRequest> {
    dispatch_for_events(
        tx,
        &[DomainEvent::HandCompareFinished {
            player: owner,
            target,
            player_card: CardId::NONE,
            target_card: CardId::NONE,
            winner,
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap()
}

#[test]
fn package_win_lock_and_non_win_recovery_are_exact() {
    let package = content_compiler::compile_source_named("yijue-test", SOURCE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 356);
    let active = package
        .active_index
        .iter()
        .find(|active| active.skill_def_id == YIJUE)
        .unwrap();
    assert_eq!(active.uses_per_turn, Some(1));
    assert!(active.targets.as_ref().unwrap().can_compare);

    let (state, owner, target) = setup();
    let state = add_skill(add_skill(state, owner, YIJUE_WIN), target, YIJUE);
    let mut tx = Transaction::begin(state);
    let effects = compare_effects(&mut tx, owner, target, owner);
    run_effects(&mut tx, effects).unwrap();
    assert!(tx
        .state()
        .turn
        .non_locked_skill_suppressed_players
        .contains(&target));
    let target_skills: Vec<_> = tx
        .state()
        .players
        .get(target)
        .unwrap()
        .skills
        .iter()
        .filter_map(|skill| tx.state().skills.get(*skill))
        .collect();
    assert!(target_skills.iter().any(|skill| skill.def.raw() == YIJUE2));
    assert!(target_skills
        .iter()
        .find(|skill| skill.def.raw() == YIJUE)
        .is_some_and(|skill| !tx.state().skill_effectively_enabled(skill)));

    let (mut state, owner, target) = setup();
    state.players.get_mut(target).unwrap().hp -= 1;
    let state = add_skill(state, owner, YIJUE_LOSE);
    let mut tx = Transaction::begin(state);
    let effects = compare_effects(&mut tx, owner, target, target);
    assert_eq!(
        run_effects(&mut tx, effects),
        Err(EffectError::ChoiceRequested)
    );
    let hp = tx.state().players.get(target).unwrap().hp;
    let state = tx.commit().state;
    let (state, result) = apply_command(
        state.clone(),
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(state.revision),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::ResolveChoice {
                choice_id: ChoiceId::new(state.resolution.pending_choice_id),
                selections: vec![2],
                cancelled: false,
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(target).unwrap().hp, hp + 1);
}
