//! Golden coverage for 四象「士仁」.

use content_compiler::compile_source_named;
use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, ContinuationKind, GameState, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const STDSHIREN: u32 = 2_834_902_877;
const SOURCE: &str = include_str!("../../../../content/original/ported/sixiang/stdshiren.skill");

fn setup() -> (GameState, PlayerId, PlayerId) {
    let package = compile_source_named("stdshiren-test", SOURCE).unwrap();
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content)
        .load_package(&package)
        .unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 2, 20_260_715, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let owner = players[0];
    let source = players[1];
    let old_skills: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    run_effects(&mut tx, old_skills).unwrap();
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(STDSHIREN),
        }],
    )
    .unwrap();
    (tx.commit().state, owner, source)
}

fn inject(state: &mut GameState, owner: PlayerId, kind: CardKind, rank: u8) -> CardId {
    let hand = state.zones.find(ZoneKind::Hand, owner).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(9_800 + u32::from(rank)),
        kind,
        suit: Suit::Spade,
        rank,
        weapon_range: 0,
        zone: hand,
        owner,
        face_up: false,
    });
    state.zones.get_mut(hand).unwrap().cards.push(card);
    card
}

fn resolve(
    state: GameState,
    actor: PlayerId,
    selection: u32,
    key: u64,
) -> (GameState, PipelineResult) {
    let choice = ChoiceId::new(state.resolution.pending_choice_id);
    apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id: choice,
                selections: vec![selection],
                cancelled: false,
            },
        },
    )
}

fn use_on(
    state: GameState,
    source: PlayerId,
    target: PlayerId,
    card: CardId,
) -> (GameState, Result<Vec<engine_api::DomainEvent>, EffectError>) {
    let mut tx = Transaction::begin(state);
    let result = run_effects(
        &mut tx,
        vec![EffectRequest::UseCard {
            player: source,
            card,
            targets: vec![target],
        }],
    );
    (tx.commit().state, result)
}

#[test]
fn only_the_first_other_players_sha_each_turn_draws_two_then_gives_one() {
    let (mut state, owner, source) = setup();
    let non_sha = inject(&mut state, source, CardKind::Other, 5);
    let (mut state, result) = use_on(state, source, owner, non_sha);
    assert!(result.is_ok());
    assert!(!state.has_pending_choice());

    let old_card = state.hand_cards(owner)[0];
    let owner_hand = state.hand_count(owner);
    let source_hand = state.hand_count(source);
    let sha = inject(&mut state, source, CardKind::Sha, 6);
    let (state, result) = use_on(state, source, owner, sha);
    assert_eq!(result, Err(EffectError::ChoiceRequested));
    let (state, accepted) = resolve(state, owner, 1, 1);
    assert!(matches!(accepted.result, CommandResult::Accepted { .. }));
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::GiveOneHandOrEquip
    );
    assert_eq!(state.hand_count(owner), owner_hand + 2);

    let (mut state, given) = resolve(state, owner, old_card.raw(), 2);
    assert!(matches!(given.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(owner), owner_hand + 1);
    assert_eq!(state.hand_count(source), source_hand + 1);

    let second_sha = inject(&mut state, source, CardKind::Sha, 7);
    let owner_after = state.hand_count(owner);
    let (state, result) = use_on(state, source, owner, second_sha);
    assert!(result.is_ok());
    assert!(!state.has_pending_choice());
    assert_eq!(state.hand_count(owner), owner_after);
}
