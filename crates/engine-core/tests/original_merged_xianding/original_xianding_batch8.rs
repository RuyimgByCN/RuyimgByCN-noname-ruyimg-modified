//! Golden for XIANDING（限定包）批次八：
//! 心忧（dcxinyou）、鸟翔（dcniaoxiang）。

use engine_api::{
    ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey, PlayerId,
    SkillDefId, SkillId,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{GameState, Phase};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const DCXINYOU: u32 = 2_388_835_288;
const DCNIAOXIANG: u32 = 2_962_016_375;
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
    state: GameState,
    actor: PlayerId,
    skill: SkillId,
    key: u64,
) -> (GameState, engine_core::command::PipelineResult) {
    let revision = state.revision;
    apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ActivateSkill {
                skill,
                targets: Vec::new(),
                card_targets: Vec::new(),
            },
        },
    )
}

#[test]
fn package_exposes_batch8_skills_with_expected_contracts() {
    let package = package();
    assert!(package.manifest.rule_vm_abi_version >= 587);

    let niaoxiang = package
        .modifier_index
        .iter()
        .find(|modifier| modifier.skill_def_id == DCNIAOXIANG)
        .expect("dcniaoxiang modifier");
    assert_eq!(
        niaoxiang.target,
        "sha_response_count_if_source_owner_target_distance_le_one"
    );
    assert_eq!(niaoxiang.mode, "add");
    assert_eq!(niaoxiang.value, 1);

    let xinyou = package
        .active_index
        .iter()
        .find(|active| active.skill_def_id == DCXINYOU)
        .expect("dcxinyou active");
    assert_eq!(xinyou.window, "play_phase");
    assert_eq!(xinyou.uses_per_turn, Some(1));
}

#[test]
fn dcxinyou_recovers_and_draws_to_max_hp() {
    let (mut tx, owner, _first, _second) = setup(20_260_820);
    add_skill(&mut tx, owner, DCXINYOU);
    tx.state_mut().players.get_mut(owner).unwrap().hp = 1;
    let max_hp = tx.state().players.get(owner).unwrap().max_hp as i32;
    let skill = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .find(|skill| skill.owner == owner && skill.def.raw() == DCXINYOU)
        .expect("dcxinyou skill instance")
        .id;
    let (state, pipe) = activate(tx.state().clone(), owner, skill, 2);
    assert!(matches!(pipe.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(owner).unwrap().hp, max_hp);
    assert_eq!(state.hand_count(owner) as i32, max_hp);
}
