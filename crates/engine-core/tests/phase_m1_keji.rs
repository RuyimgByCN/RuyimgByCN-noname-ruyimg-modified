//! M1 克己（吕蒙）: skip the discard phase if no 杀 was played this turn.

use engine_api::{
    CardDefId, Command, CommandKind, CommandResult, DomainEvent, ExpectedRevision, IdempotencyKey,
    PlayerId, Suit, TurnPhaseTag,
};
use engine_core::command::apply_command;
use engine_core::effect::run_effects;
use engine_core::state::{turn_flag, Card, CardKind, Phase, ZoneKind};
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

fn inject(tx: &mut Transaction, owner: PlayerId, hand: engine_api::ZoneId, rank: u8) {
    let card = Card {
        id: engine_api::CardId::NONE,
        def: CardDefId::new(8500 + rank as u32),
        kind: CardKind::Shan,
        suit: Suit::Club,
        rank,
        weapon_range: 0,
        zone: hand,
        owner,
        face_up: false,
    };
    let cid = tx.state_mut().cards.alloc(card);
    if let Some(z) = tx.state_mut().zones.get_mut(hand) {
        z.cards.push(cid);
    }
    if let Some(c) = tx.state_mut().cards.get_mut(cid) {
        c.zone = hand;
        c.owner = owner;
    }
}

/// Grant 克己 to the current player and give an over-limit hand in Play phase.
fn setup_keji_over_limit(seed: u64) -> (engine_core::state::GameState, PlayerId) {
    let mut eng = Engine::new();
    eng.submit(start(seed, 2));
    let actor = eng.state().turn.current;
    let mut tx = Transaction::begin(eng.state().clone());
    tx.state_mut().revision = 1;
    // Sit at the Discard phase so a single EndPhase runs the discard-arm logic.
    tx.state_mut().turn.phase = Phase::Discard;
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: actor,
        skill_def: engine_api::SkillDefId::new(skill_def::KEJI),
    });
    run_effects(&mut tx, effects).unwrap();
    // Give enough hand to exceed the hp-based hand limit.
    let hand = tx.state().zones.find(ZoneKind::Hand, actor).unwrap();
    let hp = tx.state().players.get(actor).unwrap().hp.max(0) as u32;
    while tx.state().hand_count(actor) <= hp + 2 {
        let r = (tx.state().hand_count(actor) % 13 + 1) as u8;
        inject(&mut tx, actor, hand, r);
    }
    (tx.state().clone(), actor)
}

#[test]
fn keji_skips_discard_when_no_sha() {
    let (state, actor) = setup_keji_over_limit(7);
    // No 杀 played (flag unset). EndPhase: Play → Discard → (skip) → End.
    let hand_before = state.hand_count(actor);
    assert_eq!(state.turn.flag(turn_flag::SHA_PLAYED), 0);
    let (st, pipe) = apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::EndPhase,
        },
    );
    assert!(
        matches!(pipe.result, CommandResult::Accepted { .. }),
        "{:?}",
        pipe.result
    );
    assert!(
        !st.has_pending_choice(),
        "克己 should skip discard, no choice opened"
    );
    assert_eq!(st.hand_count(actor), hand_before, "no cards discarded");
    assert_ne!(st.turn.phase, Phase::Discard, "should have passed discard");
    assert_eq!(
        st.turn.skipped_phase_count(),
        1,
        "克己跳过弃牌阶段必须进入规范回合历史"
    );
    assert!(pipe.events.iter().any(|event| matches!(
        event,
        DomainEvent::PhaseSkipped {
            current,
            phase: TurnPhaseTag::Discard,
        } if *current == actor
    )));
}

#[test]
fn keji_requires_discard_when_sha_played() {
    let (mut state, actor) = setup_keji_over_limit(7);
    // Simulate that a 杀 was played this turn.
    state.turn.set_flag(turn_flag::SHA_PLAYED, 1);
    let (st, pipe) = apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::EndPhase,
        },
    );
    assert!(
        matches!(pipe.result, CommandResult::Accepted { .. }),
        "{:?}",
        pipe.result
    );
    assert!(
        st.has_pending_choice(),
        "with 杀 played, discard-to-limit choice must open"
    );
}
