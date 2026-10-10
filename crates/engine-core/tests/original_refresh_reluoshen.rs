//! Golden coverage for refresh 界洛神 rewritten from the legacy JavaScript semantics.

use content_compiler::compile_source_named;
use engine_api::{
    ChoiceId, ChoiceKind, Command, CommandKind, CommandResult, DomainEvent, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit, TurnPhaseTag,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::query::discard_to_limit_tokens;
use engine_core::state::{GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const RE_ZHENJI: u32 = 3_716_126_919;
const RELUOSHEN: u32 = 3_265_545_896;
const SOURCE: &str = include_str!("../../../content/original/ported/refresh/reluoshen.skill");
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/refresh/refresh.npack.json");

struct Setup {
    state: GameState,
    owner: PlayerId,
    black: engine_api::CardId,
}

fn setup() -> Setup {
    let package = compile_source_named("refresh-reluoshen-test", SOURCE).unwrap();
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package).unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 3, 20_260_730_309, "identity-refresh").unwrap();
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
        skill_def: SkillDefId::new(RELUOSHEN),
    });
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Prepare;
    let draw = tx
        .state()
        .zones
        .find(ZoneKind::DrawPile, PlayerId::NONE)
        .unwrap();
    let black = *tx.state().zones.get(draw).unwrap().cards.last().unwrap();
    tx.state_mut().cards.get_mut(black).unwrap().suit = Suit::Spade;
    Setup {
        state: tx.commit().state,
        owner,
        black,
    }
}

fn offer(setup: Setup) -> (GameState, PlayerId, engine_api::CardId) {
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
    (committed.state, setup.owner, setup.black)
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
fn package_closes_re_zhenji_without_legacy_javascript() {
    let compiled = compile_source_named("refresh-reluoshen", SOURCE).unwrap();
    assert_eq!(compiled.manifest.rule_vm_abi_version, 309);
    assert_eq!(
        rule_vm::bytecode::effect_tag::JUDGE_BLACK_GAIN_REPEAT_PROTECT_HAND_LIMIT,
        293
    );
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == RE_ZHENJI)
        .unwrap();
    assert_eq!(character.skills, [RELUOSHEN, 24]);
    assert!(!PACKAGE.windows(3).any(|bytes| bytes == b".js"));
}

#[test]
fn gained_black_card_is_protected_only_for_the_current_turn() {
    let (state, owner, black) = offer(setup());
    let ordinary_hand_card = state.hand_cards(owner)[0];
    let (state, repeated) = resolve(state, owner, vec![1], false, 1);
    assert_eq!(repeated.choice.as_ref().unwrap().kind, ChoiceKind::Confirm);
    assert!(state.hand_cards(owner).contains(&black));
    assert!(state.turn.hand_limit_exempt_cards.contains(&(owner, black)));
    let discardable = discard_to_limit_tokens(&state, owner);
    assert!(!discardable.contains(&black));
    assert!(discardable.contains(&ordinary_hand_card));

    let (mut state, stopped) = resolve(state, owner, Vec::new(), true, 2);
    assert!(matches!(stopped.result, CommandResult::Accepted { .. }));
    state.turn.phase = Phase::End;
    let (state, advanced) = apply_command(
        state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(3),
            kind: CommandKind::EndPhase,
        },
    );
    assert!(matches!(advanced.result, CommandResult::Accepted { .. }));
    assert!(state.turn.hand_limit_exempt_cards.is_empty());
    assert!(discard_to_limit_tokens(&state, owner).contains(&black));
}
