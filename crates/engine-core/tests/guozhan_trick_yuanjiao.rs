//! Guozhan 远交近攻 (yuanjiao) trick mapping — ADR 0621.
//!
//! 国战牌堆中的 yuanjiao 直接映射既有 CardKind::TrickYuanjiao：目标摸一张，使用者摸
//! 三张；目标策略已过滤「势力不同」的角色。

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::run_effects;
use engine_core::state::{Card, CardKind, ContinuationKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::EffectRequest;

fn inject_yuanjiao(tx: &mut Transaction, owner: PlayerId) -> CardId {
    let zone = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(468_000),
        kind: CardKind::TrickYuanjiao,
        suit: Suit::Heart,
        rank: 9,
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
fn yuanjiao_enemy_target_draws_one_and_user_draws_three() {
    let mut engine = Engine::new();
    let start = engine.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(99),
        kind: CommandKind::StartGame {
            player_count: 2,
            rng_seed: 99,
            ruleset: "guozhan-standard".into(),
        },
    });
    assert!(matches!(start, CommandResult::Accepted { .. }));
    let actor = engine.state().turn.current;
    let other = engine
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .find(|player| *player != actor)
        .expect("second player");
    let mut tx = Transaction::begin(engine.state().clone());
    tx.state_mut().group_overrides.insert(actor, "wei".into());
    tx.state_mut().group_overrides.insert(other, "shu".into());
    for p in [actor, other] {
        let cards = tx.state().hand_cards(p);
        if !cards.is_empty() {
            run_effects(&mut tx, vec![EffectRequest::DiscardCards { player: p, cards }]).unwrap();
        }
    }
    let yuanjiao = inject_yuanjiao(&mut tx, actor);
    tx.state_mut().turn.current = actor;
    tx.state_mut().turn.phase = Phase::Play;
    let state = tx.commit().state;
    let (mut state, output) = apply_command(
        state.clone(),
        Command {
            actor,
            expected_revision: ExpectedRevision(state.revision),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::PlayCard {
                card: yuanjiao,
                targets: vec![other],
                card_targets: Vec::new(),
            },
        },
    );
    assert!(
        matches!(output.result, CommandResult::Accepted { .. }),
        "yuanjiao play rejected: {:?}",
        output.result
    );
    let mut key = 2;
    while state.has_pending_choice()
        && state.resolution.continuation_kind == ContinuationKind::WuxieWindow
    {
        state = resolve(state, vec![], true, key);
        key += 1;
    }
    assert!(!state.has_pending_choice(), "yuanjiao settled");
    assert_eq!(state.hand_count(other), 1, "enemy target drew one");
    assert_eq!(state.hand_count(actor), 3, "user drew three");
}
