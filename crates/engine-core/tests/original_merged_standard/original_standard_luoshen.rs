//! Golden coverage for Standard 洛神 rewritten from the legacy JavaScript semantics.

use content_compiler::compile_source_named;
use engine_api::{
    ChoiceId, ChoiceKind, Command, CommandKind, CommandResult, DomainEvent, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit, TurnPhaseTag,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const LUOSHEN: u32 = 3_972_660_146;
const SOURCE: &str = include_str!("../../../../content/standard/luoshen.skill");
const PACKAGE: &[u8] = include_bytes!("../../../../content/standard/standard.npack.json");

struct Setup {
    state: GameState,
    owner: PlayerId,
    first_black: engine_api::CardId,
    second_black: engine_api::CardId,
    red: engine_api::CardId,
}

fn setup() -> Setup {
    let package = compile_source_named("standard-luoshen-test", SOURCE).unwrap();
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content)
        .load_package(&package)
        .unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 3, 20_260_725_182, "identity-standard").unwrap();
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
        skill_def: SkillDefId::new(LUOSHEN),
    });
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Prepare;
    let draw = tx
        .state()
        .zones
        .find(ZoneKind::DrawPile, PlayerId::NONE)
        .unwrap();
    let cards = tx.state().zones.get(draw).unwrap().cards.clone();
    let first_black = cards[cards.len() - 1];
    let second_black = cards[cards.len() - 2];
    let red = cards[cards.len() - 3];
    tx.state_mut().cards.get_mut(first_black).unwrap().suit = Suit::Spade;
    tx.state_mut().cards.get_mut(second_black).unwrap().suit = Suit::Club;
    tx.state_mut().cards.get_mut(red).unwrap().suit = Suit::Heart;
    Setup {
        state: tx.commit().state,
        owner,
        first_black,
        second_black,
        red,
    }
}

fn offer(
    setup: Setup,
) -> (
    GameState,
    PlayerId,
    engine_api::CardId,
    engine_api::CardId,
    engine_api::CardId,
) {
    let mut tx = Transaction::begin(setup.state);
    assert_eq!(
        dispatch_for_events(
            &mut tx,
            &[DomainEvent::TurnStateChanged {
                current: setup.owner,
                phase: TurnPhaseTag::Prepare,
            }],
            &mut TriggerFireRecord::new(),
        )
        .unwrap_err(),
        EffectError::ChoiceRequested
    );
    let committed = tx.commit();
    assert_eq!(
        committed.output.choice.as_ref().unwrap().kind,
        ChoiceKind::Confirm
    );
    (
        committed.state,
        setup.owner,
        setup.first_black,
        setup.second_black,
        setup.red,
    )
}

fn resolve(
    state: GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    cancelled: bool,
    key: u64,
) -> (GameState, PipelineResult) {
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    apply_command(
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
    )
}

#[test]
fn package_closes_zhenji_with_luoshen_and_qingguo() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == 7)
        .unwrap();
    assert_eq!(character.skills, [LUOSHEN, 24]);
}

#[test]
fn black_results_are_gained_and_repeat_until_the_first_red_result() {
    let setup = setup();
    let hand_before = setup.state.hand_count(setup.owner);
    let (state, owner, first_black, second_black, red) = offer(setup);
    let (state, first) = resolve(state, owner, vec![1], false, 1);
    assert_eq!(first.choice.as_ref().unwrap().kind, ChoiceKind::Confirm);
    assert!(state.hand_cards(owner).contains(&first_black));
    let (state, second) = resolve(state, owner, vec![1], false, 2);
    assert_eq!(second.choice.as_ref().unwrap().kind, ChoiceKind::Confirm);
    assert!(state.hand_cards(owner).contains(&second_black));
    let (state, stopped) = resolve(state, owner, vec![1], false, 3);
    assert!(matches!(stopped.result, CommandResult::Accepted { .. }));
    assert!(stopped.choice.is_none());
    assert_eq!(state.hand_count(owner), hand_before + 2);
    let discard = state
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    assert_eq!(state.cards.get(red).unwrap().zone, discard);
}

#[test]
fn owner_may_stop_after_a_black_result() {
    let setup = setup();
    let hand_before = setup.state.hand_count(setup.owner);
    let (state, owner, first_black, second_black, _) = offer(setup);
    let (state, first) = resolve(state, owner, vec![1], false, 4);
    assert_eq!(first.choice.as_ref().unwrap().kind, ChoiceKind::Confirm);
    let (state, stopped) = resolve(state, owner, Vec::new(), true, 5);
    assert!(matches!(stopped.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(owner), hand_before + 1);
    assert!(state.hand_cards(owner).contains(&first_black));
    assert!(!state.hand_cards(owner).contains(&second_black));
}
