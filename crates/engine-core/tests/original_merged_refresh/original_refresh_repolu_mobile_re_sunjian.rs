//! Golden coverage for REFRESH「破虏」and MOBILE 界孙坚 rewritten from legacy JavaScript.

use engine_api::{
    ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey, PlayerId,
    SkillDefId,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{ContinuationKind, GameState};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const GZYINGHUN: u32 = 2_790_748_306;
const REPOLU: u32 = 2_820_205_777;
const RE_SUNJIAN: u32 = 976_445_267;
const SOURCE: &str = include_str!("../../../../content/original/ported/refresh/repolu.skill");
const REFRESH: &[u8] =
    include_bytes!("../../../../content/original/ported/refresh/refresh.npack.json");
const MOBILE: &[u8] =
    include_bytes!("../../../../content/original/ported/mobile/mobile.npack.json");

struct Setup {
    state: GameState,
    owner: PlayerId,
    others: Vec<PlayerId>,
}

fn setup(player_count: u8, owner_index: usize) -> Setup {
    let package = content_compiler::compile_source_named("repolu-test", SOURCE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 291);
    let mut tx = Transaction::begin(GameState::empty());
    std::sync::Arc::make_mut(&mut tx.state_mut().content)
        .load_package(&package)
        .unwrap();
    bootstrap_match(&mut tx, player_count, REPOLU as u64, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let owner = players[owner_index];
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
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(REPOLU),
    });
    run_effects(&mut tx, effects).unwrap();
    Setup {
        state: tx.commit().state,
        owner,
        others: players
            .into_iter()
            .filter(|player| *player != owner)
            .collect(),
    }
}

fn kill(state: GameState, player: PlayerId, killer: PlayerId) -> GameState {
    let mut tx = Transaction::begin(state);
    assert_eq!(
        run_effects(&mut tx, vec![EffectRequest::KillPlayer { player, killer }],),
        Err(EffectError::ChoiceRequested)
    );
    assert_eq!(
        tx.state().resolution.continuation_kind,
        ContinuationKind::ChooseRepoluTargetsDraw
    );
    tx.commit().state
}

fn resolve(
    state: GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    cancelled: bool,
    key: u64,
) -> (GameState, PipelineResult) {
    apply_command(
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
    )
}

fn mark_count(state: &GameState, player: PlayerId) -> i32 {
    state
        .players
        .get(player)
        .unwrap()
        .marks
        .iter()
        .find(|(mark, _)| *mark == REPOLU)
        .map(|(_, count)| *count)
        .unwrap_or(0)
}

#[test]
fn packages_publish_forced_death_branches_and_close_re_sunjian() {
    let refresh = content_schema::ContentPackage::from_json_bytes(REFRESH).unwrap();
    let mobile = content_schema::ContentPackage::from_json_bytes(MOBILE).unwrap();
    assert!(refresh.manifest.rule_vm_abi_version >= 298);
    let triggers: Vec<_> = refresh
        .trigger_index
        .iter()
        .filter(|trigger| trigger.skill_def_id == REPOLU)
        .collect();
    assert_eq!(triggers.len(), 2);
    assert!(triggers.iter().all(|trigger| {
        trigger.event == "death"
            && !trigger.optional
            && matches!(
                trigger.filter.as_str(),
                "owner_alive_is_source" | "owner_is_target"
            )
    }));
    assert!(mobile
        .manifest
        .dependencies
        .iter()
        .any(|dependency| dependency.package_id == "refresh"));
    assert!(mobile
        .manifest
        .dependencies
        .iter()
        .any(|dependency| dependency.package_id == "shenhua"));
    assert_eq!(
        mobile
            .character_index
            .iter()
            .find(|character| character.id == RE_SUNJIAN)
            .unwrap()
            .skills,
        [GZYINGHUN, REPOLU]
    );
    assert!(!serde_json::to_string(&(refresh, mobile))
        .unwrap()
        .contains(".js"));
}

#[test]
fn cancellation_does_not_count_and_successive_kills_draw_one_then_two() {
    let setup = setup(5, 0);
    let [first, second, third, recipient] = setup.others.as_slice() else {
        panic!("expected four other players");
    };
    let state = kill(setup.state, *first, setup.owner);
    let (state, cancelled) = resolve(state, setup.owner, Vec::new(), true, 1);
    assert!(matches!(cancelled.result, CommandResult::Accepted { .. }));
    assert_eq!(mark_count(&state, setup.owner), 0);

    let state = kill(state, *second, setup.owner);
    let (state, first_use) = resolve(
        state,
        setup.owner,
        vec![recipient.raw(), setup.owner.raw()],
        false,
        2,
    );
    assert!(matches!(first_use.result, CommandResult::Accepted { .. }));
    assert_eq!(mark_count(&state, setup.owner), 1);
    assert_eq!(state.hand_count(setup.owner), 1);
    assert_eq!(state.hand_count(*recipient), 1);

    let state = kill(state, *third, setup.owner);
    let (state, second_use) = resolve(state, setup.owner, vec![recipient.raw()], false, 3);
    assert!(matches!(second_use.result, CommandResult::Accepted { .. }));
    assert_eq!(mark_count(&state, setup.owner), 2);
    assert_eq!(state.hand_count(*recipient), 3);
    assert!(!state.has_pending_choice());
}

#[test]
fn self_death_opens_exactly_one_choice_for_the_dead_owner() {
    let setup = setup(3, 1);
    let recipient = setup.others[1];
    let state = kill(setup.state, setup.owner, setup.owner);
    assert!(!state.players.get(setup.owner).unwrap().alive);
    assert!(!state.outcome.is_finished());
    let (state, resolved) = resolve(state, setup.owner, vec![recipient.raw()], false, 4);
    assert!(matches!(resolved.result, CommandResult::Accepted { .. }));
    assert_eq!(mark_count(&state, setup.owner), 1);
    assert_eq!(state.hand_count(recipient), 1);
    assert!(!state.has_pending_choice());
}
