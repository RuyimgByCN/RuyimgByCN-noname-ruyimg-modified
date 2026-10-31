//! Guozhan 火烧连营 (huoshaolianying) trick settlement — ADR 0619.
//!
//! 火烧连营（国战）：对下家及其「队列」（以下家为中心的连续同势力座次块）中所有
//! 角色造成 1 点火属性伤害。目标自动计算。

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, Suit,
};
use engine_core::command::apply_command;
use engine_core::state::{Card, CardKind, ContinuationKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::Engine;

fn inject_huoshaolianying(tx: &mut Transaction, owner: PlayerId) -> CardId {
    let zone = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(466_000),
        kind: CardKind::TrickHuoshaolianying,
        suit: Suit::Spade,
        rank: 3,
        weapon_range: 0,
        zone,
        owner,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn resolve(state: GameState, selections: Vec<u32>, cancelled: bool, key: u64) -> GameState {
    let actor = PlayerId::new(state.resolution.pending_observer);
    let (state, output) = apply_command(
        state.clone(),
        Command {
            actor,
            expected_revision: ExpectedRevision(state.revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id: ChoiceId::new(state.resolution.pending_choice_id),
                selections,
                cancelled,
            },
        },
    );
    assert!(
        matches!(output.result, CommandResult::Accepted { .. }),
        "choice resolution rejected"
    );
    state
}

#[test]
fn huoshaolianying_fires_the_next_players_queue() {
    let mut engine = Engine::new();
    let start = engine.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(99),
        kind: CommandKind::StartGame {
            player_count: 4,
            rng_seed: 99,
            ruleset: "guozhan-standard".into(),
        },
    });
    assert!(matches!(start, CommandResult::Accepted { .. }));
    let actor = engine.state().turn.current;
    let seats: Vec<_> = engine.state().players.alive_by_seat();
    let actor_index = seats.iter().position(|p| *p == actor).unwrap();
    let p1 = seats[(actor_index + 1) % 4];
    let p2 = seats[(actor_index + 2) % 4];
    let p3 = seats[(actor_index + 3) % 4];
    let mut tx = Transaction::begin(engine.state().clone());
    // 势力：actor=wei, p1=shu, p2=shu, p3=wei。
    // 下家 p1 的队列 = {p1, p2}（连续同势力块），目标为二者。
    tx.state_mut().group_overrides.insert(actor, "wei".into());
    tx.state_mut().group_overrides.insert(p1, "shu".into());
    tx.state_mut().group_overrides.insert(p2, "shu".into());
    tx.state_mut().group_overrides.insert(p3, "wei".into());
    let huoshaolianying = inject_huoshaolianying(&mut tx, actor);
    tx.state_mut().turn.current = actor;
    tx.state_mut().turn.phase = Phase::Play;
    let state = tx.commit().state;
    let hp0 = state.players.get(actor).unwrap().hp;
    let hp1 = state.players.get(p1).unwrap().hp;
    let hp2 = state.players.get(p2).unwrap().hp;
    let hp3 = state.players.get(p3).unwrap().hp;

    let (mut state, output) = apply_command(
        state.clone(),
        Command {
            actor,
            expected_revision: ExpectedRevision(state.revision),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::PlayCard {
                card: huoshaolianying,
                targets: vec![],
                card_targets: Vec::new(),
            },
        },
    );
    assert!(
        matches!(output.result, CommandResult::Accepted { .. }),
        "huoshaolianying play rejected: {:?}",
        output.result
    );
    let mut key = 2;
    while state.has_pending_choice()
        && state.resolution.continuation_kind == ContinuationKind::WuxieWindow
    {
        state = resolve(state, vec![], true, key);
        key += 1;
    }
    assert!(!state.has_pending_choice(), "fire damage is instant");
    assert_eq!(
        state.players.get(actor).unwrap().hp,
        hp0,
        "user (different faction) untouched"
    );
    assert_eq!(
        state.players.get(p1).unwrap().hp,
        hp1 - 1,
        "next player burnt"
    );
    assert_eq!(
        state.players.get(p2).unwrap().hp,
        hp2 - 1,
        "queue member burnt"
    );
    assert_eq!(
        state.players.get(p3).unwrap().hp,
        hp3,
        "out-of-queue wei player untouched"
    );
}
