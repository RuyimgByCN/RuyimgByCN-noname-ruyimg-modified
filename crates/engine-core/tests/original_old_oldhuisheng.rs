//! Golden coverage for old「贿生」rewritten from legacy JavaScript semantics.

use engine_api::{
    ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey, PlayerId,
    SkillDefId,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::GameState;
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const OLDHUISHENG: u32 = 2_369_548_046;
const SOURCE: &str = include_str!("../../../content/original/ported/old/oldhuisheng.skill");
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/old/old.npack.json");

struct Setup {
    state: GameState,
    owner: PlayerId,
    source: PlayerId,
}

fn setup() -> Setup {
    let package = content_compiler::compile_source_named("oldhuisheng-test", SOURCE).unwrap();
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package).unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 2, 2_026_072_812, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let owner = players[0];
    let source = players[1];
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(OLDHUISHENG),
    });
    run_effects(&mut tx, effects).unwrap();
    Setup {
        state: tx.commit().state,
        owner,
        source,
    }
}

fn resolve(
    state: GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    cancelled: bool,
    key: u64,
) -> GameState {
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    let (state, result) = apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(0),
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

fn open(setup: &Setup) -> GameState {
    let mut tx = Transaction::begin(setup.state.clone());
    assert!(matches!(
        run_effects(
            &mut tx,
            vec![EffectRequest::DealDamage {
                source: setup.source,
                target: setup.owner,
                amount: 1,
                is_lose_hp: false,
                nature: rule_api::damage_nature::NORMAL,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    ));
    resolve(tx.commit().state, setup.owner, vec![1], false, 1)
}

#[test]
fn source_can_gain_to_prevent_and_is_then_recorded_for_the_match() {
    let setup = setup();
    let hp = setup.state.players.get(setup.owner).unwrap().hp;
    let source_hand = setup.state.hand_count(setup.source);
    let offered = open(&setup);
    let cards = setup.state.hand_cards(setup.owner)[..2].to_vec();
    let state = resolve(
        offered,
        setup.owner,
        cards.iter().map(|card| card.raw()).collect(),
        false,
        2,
    );
    let state = resolve(state, setup.source, vec![cards[0].raw()], false, 3);
    assert_eq!(state.players.get(setup.owner).unwrap().hp, hp);
    assert_eq!(state.hand_count(setup.source), source_hand + 1);
    let mut tx = Transaction::begin(state);
    run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source: setup.source,
            target: setup.owner,
            amount: 1,
            is_lose_hp: false,
            nature: rule_api::damage_nature::NORMAL,
        }],
    )
    .unwrap();
    assert_eq!(tx.state().players.get(setup.owner).unwrap().hp, hp - 1);
}

#[test]
fn source_can_discard_equal_count_and_damage_continues() {
    let setup = setup();
    let hp = setup.state.players.get(setup.owner).unwrap().hp;
    let owner_card = setup.state.hand_cards(setup.owner)[0];
    let source_card = setup.state.hand_cards(setup.source)[0];
    let state = resolve(open(&setup), setup.owner, vec![owner_card.raw()], false, 10);
    let state = resolve(state, setup.source, Vec::new(), true, 11);
    let state = resolve(state, setup.source, vec![source_card.raw()], false, 12);
    assert_eq!(state.players.get(setup.owner).unwrap().hp, hp - 1);
}

#[test]
fn package_publishes_oldhuisheng_without_javascript() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 275);
    assert!(package
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == OLDHUISHENG && skill.name == "oldhuisheng"));
    assert!(!serde_json::to_string(&package).unwrap().contains(".js"));
}
