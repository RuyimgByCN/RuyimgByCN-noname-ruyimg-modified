//! Golden for XIANDING（限定包）批次四：
//! 伪涕（dcweiti）及安境索引合同。

use engine_api::{
    ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey, PlayerId,
    SkillDefId, SkillId,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{ContinuationKind, GameState, Phase};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const DCANJING: u32 = 2_716_085_756;
const DCWEITI: u32 = 2_856_634_800;
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

fn activate(
    state: GameState,
    actor: PlayerId,
    skill: SkillId,
    targets: Vec<PlayerId>,
    key: u64,
) -> (GameState, PipelineResult) {
    let revision = state.revision;
    apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ActivateSkill {
                skill,
                targets,
                card_targets: Vec::new(),
            },
        },
    )
}

#[test]
fn package_exposes_batch4_skills_with_expected_contracts() {
    let package = package();
    assert!(package.manifest.rule_vm_abi_version >= 587);

    let anjing = package
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == DCANJING)
        .expect("dcanjing trigger");
    assert_eq!(anjing.event, "damage");
    assert_eq!(anjing.filter, "owner_source_wounded_exists_once_per_turn");
    assert!(anjing.optional);

    let weiti = package
        .active_index
        .iter()
        .find(|active| active.skill_def_id == DCWEITI)
        .expect("dcweiti active");
    assert_eq!(weiti.window, "play_phase");
    assert_eq!(weiti.uses_per_turn, Some(2));
}

#[test]
fn dcweiti_choose_damage_branch_damages_other() {
    let (mut tx, owner, first, _second) = setup(20_260_809);
    add_skill(&mut tx, owner, DCWEITI);
    let skill = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .find(|skill| skill.owner == owner && skill.def.raw() == DCWEITI)
        .expect("dcweiti skill instance")
        .id;
    let first_hp = tx.state().players.get(first).unwrap().hp;
    let (state, pipe) = activate(tx.state().clone(), owner, skill, vec![first], 1);
    assert!(matches!(pipe.result, CommandResult::Accepted { .. }));
    let state = resolve(state, owner, vec![0], false, 2);
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::ChooseOtherDamage
    );
    let state = resolve(state, owner, vec![first.raw()], false, 3);
    assert_eq!(state.players.get(first).unwrap().hp, first_hp - 1);
}

#[test]
fn dcweiti_choose_recover_branch_recovers_other() {
    let (mut tx, owner, first, _second) = setup(20_260_810);
    add_skill(&mut tx, owner, DCWEITI);
    tx.state_mut().players.get_mut(first).unwrap().hp -= 1;
    let skill = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .find(|skill| skill.owner == owner && skill.def.raw() == DCWEITI)
        .expect("dcweiti skill instance")
        .id;
    let first_hp = tx.state().players.get(first).unwrap().hp;
    let (state, pipe) = activate(tx.state().clone(), owner, skill, vec![first], 3);
    assert!(matches!(pipe.result, CommandResult::Accepted { .. }));
    let state = resolve(state, owner, vec![1], false, 4);
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::ChooseOtherDrawOrRecover
    );
    let state = resolve(state, owner, vec![first.raw()], false, 5);
    assert_eq!(state.players.get(first).unwrap().hp, first_hp + 1);
}
