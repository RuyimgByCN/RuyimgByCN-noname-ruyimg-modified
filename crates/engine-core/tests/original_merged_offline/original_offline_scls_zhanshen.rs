//! Golden coverage for SCL 吕布「战神」rewritten from legacy JavaScript semantics.

use engine_api::{
    ChoiceId, ChoiceKind, Command, CommandKind, CommandResult, DomainEvent, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, TurnPhaseTag,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{ContinuationKind, GameState, Phase};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const SCLS_ZHANSHEN: u32 = 3_534_164_013;
const ZHANSHEN_DRAW: u32 = 2_175_496_411;
const ZHANSHEN_DAMAGE: u32 = 2_364_944_790;
const ZHANSHEN_TARGET: u32 = 2_535_182_519;
const SCL_LVBU: u32 = 2_234_422_739;
const SOURCE: &str =
    include_str!("../../../../content/original/ported/offline/scls_zhanshen.skill");
const PACKAGE: &[u8] =
    include_bytes!("../../../../content/original/ported/offline/offline.npack.json");

fn has_skill(state: &GameState, owner: PlayerId, def: u32) -> bool {
    state
        .players
        .get(owner)
        .unwrap()
        .skills
        .iter()
        .any(|skill| {
            state
                .skills
                .get(*skill)
                .is_some_and(|instance| instance.def.raw() == def)
        })
}

fn setup() -> (Transaction, PlayerId) {
    let package = content_compiler::compile_source_named("scls-zhanshen-test", SOURCE).unwrap();
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content)
        .load_package(&package)
        .unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 2, SCLS_ZHANSHEN as u64, "identity-standard").unwrap();
    let owner = tx.state().players.alive_by_seat()[0];
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(SCLS_ZHANSHEN),
    });
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Prepare;
    (tx, owner)
}

fn fire_prepare(mut tx: Transaction, owner: PlayerId) -> (GameState, Vec<u32>) {
    let effects = dispatch_for_events(
        &mut tx,
        &[DomainEvent::TurnStateChanged {
            current: owner,
            phase: TurnPhaseTag::Prepare,
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    let result = run_effects(&mut tx, effects);
    if tx.state().has_pending_choice() {
        assert_eq!(result, Err(EffectError::ChoiceRequested));
        assert_eq!(
            tx.state().resolution.continuation_kind,
            ContinuationKind::ChooseAddMissingSkill
        );
    } else {
        result.unwrap();
    }
    let committed = tx.commit();
    let options = committed
        .output
        .choice
        .map(|choice| {
            assert_eq!(choice.kind, ChoiceKind::Custom);
            assert!(!choice.allow_cancel);
            choice
                .options
                .into_iter()
                .map(|option| option.token)
                .collect()
        })
        .unwrap_or_default();
    (committed.state, options)
}

fn choose(state: GameState, owner: PlayerId, skill_def: u32, key: u64) -> GameState {
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    let (state, result) = apply_command(
        state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections: vec![skill_def],
                cancelled: false,
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    state
}

#[test]
fn package_publishes_skill_helpers_and_complete_character() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let compiled = content_compiler::compile_source_named("scls_zhanshen", SOURCE).unwrap();
    assert_eq!(compiled.manifest.rule_vm_abi_version, 79);
    assert_eq!(package.manifest.rule_vm_abi_version, 691);
    for (def, name) in [
        (SCLS_ZHANSHEN, "scls_zhanshen"),
        (ZHANSHEN_DRAW, "scls_zhanshen_draw"),
        (ZHANSHEN_DAMAGE, "scls_zhanshen_damage"),
        (ZHANSHEN_TARGET, "scls_zhanshen_target"),
    ] {
        assert!(package
            .skill_index
            .iter()
            .any(|skill| skill.skill_def_id == def && skill.name == name && skill.locked));
    }
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == SCL_LVBU)
        .unwrap();
    assert_eq!(character.skills, [SCLS_ZHANSHEN]);
    let expected = compiled
        .modules
        .iter()
        .find(|module| module.name == "scls_zhanshen_phase_prepare")
        .unwrap();
    let actual = package
        .modules
        .iter()
        .find(|module| module.name == expected.name)
        .unwrap();
    assert_eq!(actual.logic_hash, expected.logic_hash);
}

#[test]
fn each_prepare_adds_one_missing_permanent_bonus_and_last_is_automatic() {
    let (tx, owner) = setup();
    let base_draw = engine_core::query::draw_count_for(tx.state(), owner);

    let (state, options) = fire_prepare(tx, owner);
    assert_eq!(options, [ZHANSHEN_DRAW, ZHANSHEN_DAMAGE, ZHANSHEN_TARGET]);
    let state = choose(state, owner, ZHANSHEN_DAMAGE, 1);
    assert!(has_skill(&state, owner, ZHANSHEN_DAMAGE));
    assert_eq!(engine_core::query::sha_damage_for(&state, owner, 1), 2);

    let (state, options) = fire_prepare(Transaction::begin(state), owner);
    assert_eq!(options, [ZHANSHEN_DRAW, ZHANSHEN_TARGET]);
    let state = choose(state, owner, ZHANSHEN_TARGET, 2);
    assert!(has_skill(&state, owner, ZHANSHEN_TARGET));
    assert_eq!(engine_core::query::sha_target_limit_for(&state, owner), 2);

    let (state, options) = fire_prepare(Transaction::begin(state), owner);
    eprintln!("DBG prepare3 options: {:?}", options);
    eprintln!(
        "DBG owner skills: {:?}",
        state
            .players
            .get(owner)
            .unwrap()
            .skills
            .iter()
            .collect::<Vec<_>>()
    );
    assert!(options.is_empty());
    assert!(has_skill(&state, owner, ZHANSHEN_DRAW));
    assert_eq!(
        engine_core::query::draw_count_for(&state, owner),
        base_draw + 1
    );

    let (state, options) = fire_prepare(Transaction::begin(state), owner);
    eprintln!("DBG prepare3 options: {:?}", options);
    eprintln!(
        "DBG owner skills: {:?}",
        state
            .players
            .get(owner)
            .unwrap()
            .skills
            .iter()
            .collect::<Vec<_>>()
    );
    assert!(options.is_empty());
    assert_eq!(
        engine_core::query::draw_count_for(&state, owner),
        base_draw + 1
    );
}
