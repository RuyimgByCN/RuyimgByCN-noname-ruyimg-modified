//! M1 观星（诸葛亮）: peek top N of the draw pile and reorder at turn start.

use engine_api::{Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey, PlayerId};
use engine_core::command::apply_command;
use engine_core::effect::run_effects;
use engine_core::state::{ContinuationKind, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::{content::skill_def, Engine};
use rule_api::EffectRequest;

fn start(seed: u64, n: u8) -> Command {
    Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(0),
        kind: CommandKind::StartGame {
            player_count: n,
            rng_seed: seed,
            ruleset: "identity-standard".into(),
        },
    }
}

/// Build a game where the current player has 观星 and sits in Prepare phase.
/// Returns (state, actor, top_cards_bottom_to_top).
fn setup(seed: u64) -> (engine_core::state::GameState, PlayerId, Vec<u32>) {
    let mut eng = Engine::new();
    eng.submit(start(seed, 2));
    let actor = eng.state().turn.current;
    let mut tx = Transaction::begin(eng.state().clone());
    tx.state_mut().revision = 1;
    tx.state_mut().turn.phase = Phase::Prepare;
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: actor,
            skill_def: engine_api::SkillDefId::new(skill_def::GUANXING),
        }],
    )
    .unwrap();
    let draw = tx
        .state()
        .zones
        .find(ZoneKind::DrawPile, PlayerId::NONE)
        .unwrap();
    let cards: Vec<u32> = tx
        .state()
        .zones
        .get(draw)
        .unwrap()
        .cards
        .iter()
        .map(|c| c.raw())
        .collect();
    (tx.state().clone(), actor, cards)
}

fn end_phase(
    state: engine_core::state::GameState,
    actor: PlayerId,
    key: u64,
) -> (
    engine_core::state::GameState,
    engine_core::command::PipelineResult,
) {
    apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::EndPhase,
        },
    )
}

#[test]
fn guanxing_opens_choice_at_prepare() {
    let (state, actor, _cards) = setup(7);
    let (st, pipe) = end_phase(state, actor, 1);
    assert!(
        matches!(pipe.result, CommandResult::Accepted { .. }),
        "{:?}",
        pipe.result
    );
    assert!(st.has_pending_choice(), "观星 should open a reorder choice");
    assert_eq!(
        st.resolution.continuation_kind,
        ContinuationKind::ReorderDrawPile
    );
    // N = min(alive=2, 5) = 2 tokens offered.
    let choice = pipe.choice.expect("choice");
    assert_eq!(choice.options.len(), 2, "should peek 2 cards (2 players)");
}

#[test]
fn guanxing_cancel_keeps_order() {
    let (state, actor, cards_before) = setup(7);
    let (st, pipe) = end_phase(state, actor, 1);
    let choice = pipe.choice.expect("choice");
    // Cancel: keep all on top, order unchanged.
    let (st2, pipe2) = apply_command(
        st,
        Command {
            actor,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(2),
            kind: CommandKind::ResolveChoice {
                choice_id: choice.choice_id,
                selections: vec![],
                cancelled: true,
            },
        },
    );
    assert!(
        matches!(pipe2.result, CommandResult::Accepted { .. }),
        "{:?}",
        pipe2.result
    );
    let draw = st2.zones.find(ZoneKind::DrawPile, PlayerId::NONE).unwrap();
    let cards_after: Vec<u32> = st2
        .zones
        .get(draw)
        .unwrap()
        .cards
        .iter()
        .map(|c| c.raw())
        .collect();
    assert_eq!(
        cards_after, cards_before,
        "cancel must not reorder the pile"
    );
    assert_eq!(
        st2.turn.phase,
        Phase::Judge,
        "should advance to Judge after 观星"
    );
}

#[test]
fn guanxing_sinks_selected_to_bottom() {
    let (state, actor, cards_before) = setup(7);
    let top_card = *cards_before.last().unwrap(); // top of deck (drawn first)
    let (st, pipe) = end_phase(state, actor, 1);
    let choice = pipe.choice.expect("choice");
    // Send the very top card to the bottom.
    let (st2, pipe2) = apply_command(
        st,
        Command {
            actor,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(2),
            kind: CommandKind::ResolveChoice {
                choice_id: choice.choice_id,
                selections: vec![top_card],
                cancelled: false,
            },
        },
    );
    assert!(
        matches!(pipe2.result, CommandResult::Accepted { .. }),
        "{:?}",
        pipe2.result
    );
    let draw = st2.zones.find(ZoneKind::DrawPile, PlayerId::NONE).unwrap();
    let cards_after: Vec<u32> = st2
        .zones
        .get(draw)
        .unwrap()
        .cards
        .iter()
        .map(|c| c.raw())
        .collect();
    assert_eq!(cards_after.len(), cards_before.len(), "no cards lost");
    assert_eq!(
        *cards_after.first().unwrap(),
        top_card,
        "selected card sank to bottom"
    );
    assert_ne!(*cards_after.last().unwrap(), top_card, "top card changed");
    // The card that was 2nd-from-top is now the top.
    let second = cards_before[cards_before.len() - 2];
    assert_eq!(
        *cards_after.last().unwrap(),
        second,
        "next card surfaced to top"
    );
}
