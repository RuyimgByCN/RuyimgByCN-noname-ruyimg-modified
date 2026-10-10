//! Golden coverage for SP2 牛金「摧锐」rewritten from legacy JavaScript semantics.

use engine_api::{
    ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey, PlayerId,
    SkillDefId, SkillId,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::hash::canonical_state_hash;
use engine_core::state::{GameState, Phase};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const RE_NIUJIN: u32 = 3_368_551_416;
const RECUORUI: u32 = 4_246_379_136;
const RELIEWEI: u32 = 3_943_534_187;
const SOURCE: &str = include_str!("../../../content/original/ported/sp2/recuorui.skill");
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/sp2/sp2.npack.json");

fn setup(hp: i32) -> (GameState, PlayerId, Vec<PlayerId>, SkillId) {
    let package = content_compiler::compile_source_named("recuorui-test", SOURCE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 288);
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package).unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 3, 20_260_729_31, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let owner = players[0];
    let targets = players[1..].to_vec();
    let effects = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .chain([EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(RECUORUI),
        }])
        .collect();
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    tx.state_mut().players.get_mut(owner).unwrap().hp = hp;
    let skill = tx
        .state()
        .players
        .get(owner)
        .unwrap()
        .skills
        .iter()
        .copied()
        .find(|skill| tx.state().skills.get(*skill).unwrap().def.raw() == RECUORUI)
        .unwrap();
    (tx.commit().state, owner, targets, skill)
}

fn activate(
    state: GameState,
    owner: PlayerId,
    skill: SkillId,
    targets: Vec<PlayerId>,
    key: u64,
) -> (GameState, engine_core::command::PipelineResult) {
    apply_command(
        state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ActivateSkill {
                skill,
                targets,
                card_targets: Vec::new(),
            },
        },
    )
}

fn resolve(
    state: GameState,
    owner: PlayerId,
    card: u32,
    key: u64,
) -> (GameState, engine_core::command::PipelineResult) {
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    apply_command(
        state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections: vec![card],
                cancelled: false,
            },
        },
    )
}

#[test]
fn package_contains_character_and_hp_capped_active_contract() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 753);
    assert_eq!(
        package
            .character_index
            .iter()
            .find(|character| character.id == RE_NIUJIN)
            .unwrap()
            .skills,
        [RECUORUI, RELIEWEI]
    );
    let active = package
        .active_index
        .iter()
        .find(|active| active.skill_def_id == RECUORUI)
        .unwrap();
    let targets = active.targets.as_ref().unwrap();
    assert!(
        active.once_per_game
            && targets.min == 1
            && targets.max == u8::MAX
            && !targets.allow_self
            && targets.max_by_owner_hp
            && targets.hand_nonempty
    );
}

#[test]
fn gains_in_submitted_order_obeys_current_hp_and_is_limited() {
    let (state, owner, targets, skill) = setup(2);
    let submitted = vec![targets[1], targets[0]];
    let mut first_options: Vec<_> = state
        .hand_cards(submitted[0])
        .into_iter()
        .map(|card| card.raw())
        .collect();
    first_options.sort_unstable();
    let mut second_options: Vec<_> = state
        .hand_cards(submitted[1])
        .into_iter()
        .map(|card| card.raw())
        .collect();
    second_options.sort_unstable();

    let (state, pipe) = activate(state, owner, skill, submitted.clone(), 1);
    assert!(
        matches!(pipe.result, CommandResult::Accepted { .. }),
        "{:?}",
        pipe.result
    );
    let offered: Vec<_> = pipe
        .choice
        .unwrap()
        .options
        .into_iter()
        .map(|option| option.token)
        .collect();
    assert_eq!(offered, first_options);

    let first = first_options[0];
    let (state, pipe) = resolve(state, owner, first, 2);
    assert!(matches!(pipe.result, CommandResult::Accepted { .. }));
    assert!(state
        .hand_cards(owner)
        .iter()
        .any(|card| card.raw() == first));
    let offered: Vec<_> = pipe
        .choice
        .unwrap()
        .options
        .into_iter()
        .map(|option| option.token)
        .collect();
    assert_eq!(offered, second_options);

    let second = second_options[0];
    let (state, pipe) = resolve(state, owner, second, 3);
    assert!(matches!(pipe.result, CommandResult::Accepted { .. }));
    assert!(!state.has_pending_choice());
    assert!(state
        .hand_cards(owner)
        .iter()
        .any(|card| card.raw() == second));
    let before = canonical_state_hash(&state);
    let (same, pipe) = activate(state, owner, skill, vec![submitted[0]], 4);
    assert!(matches!(pipe.result, CommandResult::Rejected { .. }));
    assert_eq!(canonical_state_hash(&same), before);

    let (state, owner, targets, skill) = setup(1);
    let before = canonical_state_hash(&state);
    let (same, pipe) = activate(state, owner, skill, targets, 5);
    assert!(matches!(pipe.result, CommandResult::Rejected { .. }));
    assert_eq!(canonical_state_hash(&same), before);
}
