//! Golden for XIANDING（限定包）批次二十一：
//! 守户（dcshouhu）、烈骧（dcliexiang）。

use engine_api::{
    ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey, PlayerId,
    SkillDefId,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{ContinuationKind, GameState, Phase};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const DCSHOUHU: u32 = 3_777_360_870;
const DCLIEXIANG: u32 = 3_778_727_333;
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

fn activate(
    tx: &Transaction,
    actor: PlayerId,
    def: u32,
    targets: Vec<PlayerId>,
    key: u64,
) -> GameState {
    let skill_id = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .find(|skill| skill.def == SkillDefId::new(def))
        .expect("live skill")
        .id;
    let (state, result) = apply_command(
        tx.state().clone(),
        Command {
            actor,
            expected_revision: ExpectedRevision(tx.state().revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ActivateSkill {
                skill: skill_id,
                targets,
                card_targets: vec![],
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    state
}

#[test]
fn package_exposes_batch21_skills_with_expected_contracts() {
    let package = package();
    assert!(package.manifest.rule_vm_abi_version >= 587);

    let shouhu = package
        .active_index
        .iter()
        .find(|active| active.skill_def_id == DCSHOUHU)
        .expect("dcshouhu active");
    assert_eq!(shouhu.window, "play_phase");
    assert_eq!(shouhu.uses_per_turn, Some(1));
    assert!(shouhu.targets.as_ref().is_some_and(|t| !t.allow_self));

    let liexiang = package
        .active_index
        .iter()
        .find(|active| active.skill_def_id == DCLIEXIANG)
        .expect("dcliexiang active");
    assert_eq!(liexiang.window, "play_phase");
    assert_eq!(liexiang.uses_per_turn, Some(1));
}

#[test]
fn dcshouhu_uses_virtual_sha_on_other() {
    let (mut tx, owner, first, _second) = setup(20_260_849);
    add_skill(&mut tx, owner, DCSHOUHU);
    let state = activate(&tx, owner, DCSHOUHU, vec![first], 1);
    let mut tx = Transaction::begin(state);
    assert_eq!(
        tx.state().resolution.continuation_kind,
        ContinuationKind::VirtualShaTargets
    );
    let state = resolve(tx.state().clone(), owner, vec![first.raw()], false, 2);
    let _ = state;
}

#[test]
fn dcliexiang_draws_five_then_damages_target() {
    let (mut tx, owner, first, _second) = setup(20_260_850);
    add_skill(&mut tx, owner, DCLIEXIANG);
    let before = tx.state().hand_count(owner);
    let hp = tx.state().players.get(first).unwrap().hp;

    let state = activate(&tx, owner, DCLIEXIANG, vec![first], 1);
    assert_eq!(state.hand_count(owner), before + 5);
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::ChooseTargetsDamage
    );
    let state = resolve(state, owner, vec![first.raw()], false, 2);
    assert_eq!(state.players.get(first).unwrap().hp, hp - 1);
}
