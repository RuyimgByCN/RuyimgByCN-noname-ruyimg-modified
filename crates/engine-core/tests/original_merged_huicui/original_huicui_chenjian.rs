//! Golden tests for HUICUI「陈见」rewritten from the legacy JavaScript semantics.

use engine_api::{
    ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey, MarkDefId,
    PlayerId, SkillDefId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{run_effects, EffectError};
use engine_core::hash::canonical_state_hash;
use engine_core::state::{CardKind, ContinuationKind, GameState, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::EffectRequest;

const CHENJIAN: u32 = 3_452_524_123;

fn started_state(seed: u64) -> GameState {
    let mut engine = Engine::new();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(seed),
            kind: CommandKind::StartGame {
                player_count: 2,
                rng_seed: seed,
                ruleset: "identity-standard".into(),
            },
        }),
        CommandResult::Accepted { .. }
    ));
    engine.state().clone()
}

fn resolve(state: GameState, selections: Vec<u32>, cancelled: bool) -> (GameState, PipelineResult) {
    let actor = PlayerId::new(state.resolution.pending_observer);
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections,
                cancelled,
            },
        },
    )
}

fn mark_count(state: &GameState, player: PlayerId) -> i32 {
    state
        .players
        .get(player)
        .unwrap()
        .marks
        .iter()
        .find(|(mark, _)| *mark == CHENJIAN)
        .map_or(0, |(_, count)| *count)
}

fn set_top_three(
    tx: &mut Transaction,
    first: (CardKind, Suit),
    second: (CardKind, Suit),
    third: (CardKind, Suit),
) -> [engine_api::CardId; 3] {
    let draw = tx
        .state()
        .zones
        .find(ZoneKind::DrawPile, PlayerId::NONE)
        .unwrap();
    let pile = &tx.state().zones.get(draw).unwrap().cards;
    let cards = [
        pile[pile.len() - 1],
        pile[pile.len() - 2],
        pile[pile.len() - 3],
    ];
    for (card, (kind, suit)) in cards.into_iter().zip([first, second, third]) {
        let state = tx.state_mut().cards.get_mut(card).unwrap();
        state.kind = kind;
        state.suit = suit;
    }
    cards
}

fn start_effect(tx: &mut Transaction, owner: PlayerId) -> Result<(), EffectError> {
    run_effects(
        tx,
        vec![EffectRequest::RevealDiscardSuitGainOrUse {
            player: owner,
            mark: MarkDefId::new(CHENJIAN),
            base_count: 3,
            max_mark: 2,
            revealed: Vec::new(),
            completed: 0,
            step: 0,
            data: 0,
        }],
    )
    .map(|_| ())
}

#[test]
fn discard_first_uses_effective_discard_suit_raw_reveal_suit_and_may_skip_use() {
    const SUIT_FIXTURE: u32 = 4_294_966_997;
    let fixture = content_compiler::compile_source_named(
        "chenjian-suit-fixture",
        "skill chenjian_suit_fixture(4294966997) locked { modifier card_suit(spade) override heart; }",
    )
    .unwrap();
    let mut engine = Engine::new();
    engine.load_package(&fixture).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(2_026_071_501),
            kind: CommandKind::StartGame {
                player_count: 2,
                rng_seed: 2_026_071_501,
                ruleset: "identity-standard".into(),
            },
        }),
        CommandResult::Accepted { .. }
    ));
    let state = engine.state().clone();
    let owner = state.turn.current;
    let mut tx = Transaction::begin(state);
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(SUIT_FIXTURE),
        }],
    )
    .unwrap();
    let revealed = set_top_three(
        &mut tx,
        (CardKind::Shan, Suit::Heart),
        (CardKind::TrickWuzhong, Suit::Spade),
        (CardKind::Shan, Suit::Club),
    );
    let discard_card = tx.state().hand_cards(owner)[0];
    tx.state_mut().cards.get_mut(discard_card).unwrap().suit = Suit::Spade;

    assert_eq!(
        start_effect(&mut tx, owner),
        Err(EffectError::ChoiceRequested)
    );
    assert_eq!(
        tx.state().resolution.continuation_kind,
        ContinuationKind::RevealDiscardSuitGainOrUse
    );
    let processing = tx
        .state()
        .zones
        .find(ZoneKind::Processing, PlayerId::NONE)
        .unwrap();
    assert_eq!(tx.state().zones.get(processing).unwrap().cards.len(), 3);

    let state = tx.commit().state;
    let before_forgery = canonical_state_hash(&state);
    let (unchanged, rejected) = resolve(state.clone(), vec![99], false);
    assert!(matches!(rejected.result, CommandResult::Rejected { .. }));
    assert_eq!(canonical_state_hash(&unchanged), before_forgery);

    let (state, first) = resolve(state, vec![1], false);
    assert!(matches!(first.result, CommandResult::Accepted { .. }));
    let (state, discarded) = resolve(state, vec![discard_card.raw()], false);
    assert!(matches!(discarded.result, CommandResult::Accepted { .. }));
    let (state, gained) = resolve(state, vec![owner.raw()], false);
    assert!(matches!(gained.result, CommandResult::Accepted { .. }));
    assert_eq!(state.cards.get(revealed[0]).unwrap().owner, owner);

    let (state, skipped) = resolve(state, Vec::new(), true);
    assert!(matches!(skipped.result, CommandResult::Accepted { .. }));
    assert!(!state.has_pending_choice());
    assert_eq!(mark_count(&state, owner), 0);
    assert!(state.zones.get(processing).unwrap().cards.is_empty());
    let discard = state
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    assert_eq!(state.cards.get(revealed[1]).unwrap().zone, discard);
    assert_eq!(state.cards.get(revealed[2]).unwrap().zone, discard);
}

#[test]
fn used_wuzhong_resolves_wuxie_then_second_option_marks_and_recasts_all_hand() {
    let state = started_state(2_026_071_502);
    let owner = state.turn.current;
    let other = state
        .players
        .alive_by_seat()
        .into_iter()
        .find(|player| *player != owner)
        .unwrap();
    let mut tx = Transaction::begin(state);
    let wuxie = tx.state().hand_cards(other)[0];
    tx.state_mut().cards.get_mut(wuxie).unwrap().kind = CardKind::TrickWuxie;
    let revealed = set_top_three(
        &mut tx,
        (CardKind::TrickWuzhong, Suit::Spade),
        (CardKind::Shan, Suit::Heart),
        (CardKind::Shan, Suit::Club),
    );
    let discard_card = tx.state().hand_cards(owner)[0];
    let initial_hand_count = tx.state().hand_count(owner);
    tx.state_mut().cards.get_mut(discard_card).unwrap().suit = Suit::Heart;
    assert_eq!(
        start_effect(&mut tx, owner),
        Err(EffectError::ChoiceRequested)
    );

    let (state, _) = resolve(tx.commit().state, vec![2], false);
    let (state, used) = resolve(state, vec![revealed[0].raw()], false);
    assert!(matches!(used.result, CommandResult::Accepted { .. }));
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::WuxieWindow
    );

    // The engine asks every responder (owner, then the wuxie-holding other)
    // in turn; pass each wuxie window until the reveal continuation opens.
    let mut state = state;
    loop {
        assert_eq!(
            state.resolution.continuation_kind,
            ContinuationKind::WuxieWindow
        );
        let (s, passed_wuxie) = resolve(state, Vec::new(), true);
        assert!(matches!(
            passed_wuxie.result,
            CommandResult::Accepted { .. }
        ));
        state = s;
        if state.resolution.continuation_kind != ContinuationKind::WuxieWindow {
            break;
        }
    }
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::RevealDiscardSuitGainOrUse
    );
    let hand_before_second = state.hand_cards(owner);
    assert_eq!(hand_before_second.len() as u32, initial_hand_count + 2);

    let (state, _) = resolve(state, vec![discard_card.raw()], false);
    let (state, completed) = resolve(state, vec![other.raw()], false);
    assert!(matches!(completed.result, CommandResult::Accepted { .. }));
    assert!(!state.has_pending_choice());
    assert_eq!(mark_count(&state, owner), 1);
    assert_eq!(state.hand_count(owner), initial_hand_count + 1);
    let new_hand = state.hand_cards(owner);
    assert!(hand_before_second
        .iter()
        .all(|card| !new_hand.contains(card)));
    assert_eq!(state.cards.get(revealed[1]).unwrap().owner, other);
    let processing = state
        .zones
        .find(ZoneKind::Processing, PlayerId::NONE)
        .unwrap();
    assert!(state.zones.get(processing).unwrap().cards.is_empty());
}

#[test]
fn two_marks_reveal_five_and_never_grow_past_cap() {
    let state = started_state(2_026_071_503);
    let owner = state.turn.current;
    let mut tx = Transaction::begin(state);
    run_effects(
        &mut tx,
        vec![EffectRequest::AddMark {
            player: owner,
            mark: MarkDefId::new(CHENJIAN),
            delta: 2,
        }],
    )
    .unwrap();
    assert_eq!(
        start_effect(&mut tx, owner),
        Err(EffectError::ChoiceRequested)
    );
    let processing = tx
        .state()
        .zones
        .find(ZoneKind::Processing, PlayerId::NONE)
        .unwrap();
    assert_eq!(tx.state().zones.get(processing).unwrap().cards.len(), 5);
    assert_eq!(mark_count(tx.state(), owner), 2);
}

#[test]
fn a_dead_owner_still_cleans_up_tracked_processing_cards() {
    let state = started_state(2_026_071_504);
    let owner = state.turn.current;
    let mut tx = Transaction::begin(state);
    assert_eq!(
        start_effect(&mut tx, owner),
        Err(EffectError::ChoiceRequested)
    );
    let payload = tx.state().resolution.continuation_payload.clone();
    let len = payload[6] as usize;
    let revealed: Vec<_> = payload[7..7 + len]
        .iter()
        .copied()
        .map(engine_api::CardId::new)
        .collect();
    let mut state = tx.commit().state;
    state.resolution.clear_choice();
    state.players.get_mut(owner).unwrap().alive = false;
    let mut tx = Transaction::begin(state);
    run_effects(
        &mut tx,
        vec![EffectRequest::RevealDiscardSuitGainOrUse {
            player: owner,
            mark: MarkDefId::new(CHENJIAN),
            base_count: 3,
            max_mark: 2,
            revealed,
            completed: 0,
            step: 0,
            data: 0,
        }],
    )
    .unwrap();
    let processing = tx
        .state()
        .zones
        .find(ZoneKind::Processing, PlayerId::NONE)
        .unwrap();
    assert!(tx.state().zones.get(processing).unwrap().cards.is_empty());
}
