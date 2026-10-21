//! Golden coverage for OFFLINE 洛神 rewritten from the legacy JavaScript semantics.

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

const YJ_ZHENJI: u32 = 3_367_281_789;
const YJLUOSHEN: u32 = 3_100_602_929;
const SOURCE: &str = include_str!("../../../../content/original/ported/offline/yjluoshen.skill");
const PACKAGE: &[u8] =
    include_bytes!("../../../../content/original/ported/offline/offline.npack.json");

struct Setup {
    state: GameState,
    owner: PlayerId,
    first: engine_api::CardId,
    second: engine_api::CardId,
    different: engine_api::CardId,
}

fn setup(suits: [Suit; 3]) -> Setup {
    let package = compile_source_named("offline-yjluoshen-test", SOURCE).unwrap();
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content)
        .load_package(&package)
        .unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 3, 20_260_730_307, "identity-offline").unwrap();
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
        skill_def: SkillDefId::new(YJLUOSHEN),
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
    let first = cards[cards.len() - 1];
    let second = cards[cards.len() - 2];
    let different = cards[cards.len() - 3];
    for (card, suit) in [first, second, different].into_iter().zip(suits) {
        tx.state_mut().cards.get_mut(card).unwrap().suit = suit;
    }
    Setup {
        state: tx.commit().state,
        owner,
        first,
        second,
        different,
    }
}

fn offer(setup: Setup) -> Setup {
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
    Setup {
        state: committed.state,
        ..setup
    }
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
fn package_closes_yj_zhenji_without_legacy_javascript() {
    let compiled = compile_source_named("offline-yjluoshen", SOURCE).unwrap();
    assert_eq!(compiled.manifest.rule_vm_abi_version, 307);
    assert_eq!(
        rule_vm::bytecode::effect_tag::JUDGE_SAME_COLOR_GAIN_REPEAT,
        291
    );
    assert_eq!(
        engine_core::ContinuationKind::JudgeSameColorGainRepeat.tag(),
        209
    );

    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == YJ_ZHENJI)
        .unwrap();
    assert_eq!(character.skills, [YJLUOSHEN, 24]);
    assert!(!serde_json::to_string(&package).unwrap().contains(".js"));
}

#[test]
fn first_effective_color_is_gained_and_repeats_until_a_different_color() {
    for suits in [
        [Suit::Spade, Suit::Club, Suit::Heart],
        [Suit::Heart, Suit::Diamond, Suit::Spade],
    ] {
        let setup = offer(setup(suits));
        let hand_before = setup.state.hand_count(setup.owner);
        let (state, first) = resolve(setup.state, setup.owner, vec![1], false, 1);
        assert_eq!(first.choice.as_ref().unwrap().kind, ChoiceKind::Confirm);
        assert!(state.hand_cards(setup.owner).contains(&setup.first));

        let (state, second) = resolve(state, setup.owner, vec![1], false, 2);
        assert_eq!(second.choice.as_ref().unwrap().kind, ChoiceKind::Confirm);
        assert!(state.hand_cards(setup.owner).contains(&setup.second));

        let (state, stopped) = resolve(state, setup.owner, vec![1], false, 3);
        assert!(matches!(stopped.result, CommandResult::Accepted { .. }));
        assert!(stopped.choice.is_none());
        assert_eq!(state.hand_count(setup.owner), hand_before + 2);
        let discard = state
            .zones
            .find(ZoneKind::DiscardPile, PlayerId::NONE)
            .unwrap();
        assert_eq!(state.cards.get(setup.different).unwrap().zone, discard);
    }
}

#[test]
fn owner_may_stop_and_a_stale_phase_cannot_start_another_judgment() {
    let stopped_setup = offer(setup([Suit::Spade, Suit::Club, Suit::Heart]));
    let (state, first) = resolve(stopped_setup.state, stopped_setup.owner, vec![1], false, 4);
    assert_eq!(first.choice.as_ref().unwrap().kind, ChoiceKind::Confirm);
    let (state, stopped) = resolve(state, stopped_setup.owner, Vec::new(), true, 5);
    assert!(matches!(stopped.result, CommandResult::Accepted { .. }));
    assert!(!state.has_pending_choice());
    assert!(!state
        .hand_cards(stopped_setup.owner)
        .contains(&stopped_setup.second));

    let stale_setup = offer(setup([Suit::Heart, Suit::Diamond, Suit::Spade]));
    let (mut state, first) = resolve(stale_setup.state, stale_setup.owner, vec![1], false, 6);
    assert_eq!(first.choice.as_ref().unwrap().kind, ChoiceKind::Confirm);
    state.turn.phase = Phase::Play;
    let (state, stale) = resolve(state, stale_setup.owner, vec![1], false, 7);
    assert!(matches!(stale.result, CommandResult::Accepted { .. }));
    assert!(!state.has_pending_choice());
    assert!(!state
        .hand_cards(stale_setup.owner)
        .contains(&stale_setup.second));
}
