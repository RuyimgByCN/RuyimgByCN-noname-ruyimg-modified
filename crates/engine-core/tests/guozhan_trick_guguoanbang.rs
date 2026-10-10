//! Guozhan 固国安邦 (gz_guguoanbang) trick settlement — ADR 0618.
//!
//! 固国安邦（国战）：对你自己使用。摸八张牌，然后弃置至少六张手牌。若你的势力为吴，
//! 你可以将以此法弃置的牌交给其他吴势力角色（每名至多两张；单轮近似）。

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

fn inject_guguoanbang(tx: &mut Transaction, owner: PlayerId) -> CardId {
    let zone = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(465_000),
        kind: CardKind::TrickGuguoanbang,
        suit: Suit::Heart,
        rank: 1,
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

fn boot(seed: u64) -> (PlayerId, PlayerId, Transaction) {
    let mut engine = Engine::new();
    let start = engine.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(seed),
        kind: CommandKind::StartGame {
            player_count: 2,
            rng_seed: seed,
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
    (actor, other, Transaction::begin(engine.state().clone()))
}

fn clear_hand(tx: &mut Transaction, player: PlayerId) {
    let cards = tx.state().hand_cards(player);
    if !cards.is_empty() {
        run_effects(tx, vec![EffectRequest::DiscardCards { player, cards }]).unwrap();
    }
}

#[test]
fn guguoanbang_non_wu_draws_eight_discards_six() {
    let (actor, other, mut tx) = boot(99);
    tx.state_mut().group_overrides.insert(actor, "wei".into());
    tx.state_mut().group_overrides.insert(other, "wei".into());
    clear_hand(&mut tx, actor);
    let guguoanbang = inject_guguoanbang(&mut tx, actor);
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
                card: guguoanbang,
                targets: vec![actor],
                card_targets: Vec::new(),
            },
        },
    );
    assert!(
        matches!(output.result, CommandResult::Accepted { .. }),
        "guguoanbang play rejected: {:?}",
        output.result
    );
    let mut key = 2;
    while state.has_pending_choice()
        && state.resolution.continuation_kind == ContinuationKind::WuxieWindow
    {
        state = resolve(state, vec![], true, key);
        key += 1;
    }
    // 摸 8 → 手牌 8；弃 ≥6 选择挂起。
    assert!(state.has_pending_choice(), "discard choice missing");
    assert_eq!(state.hand_count(actor), 8, "drew eight");
    let hand: Vec<u32> = state.hand_cards(actor).iter().map(|c| c.raw()).collect();
    let discard6 = hand.iter().take(6).copied().collect();
    state = resolve(state, discard6, false, key);
    assert!(!state.has_pending_choice(), "non-wu settled");
    assert_eq!(state.hand_count(actor), 2, "discarded six of eight");
}

#[test]
fn guguoanbang_wu_gives_discarded_cards_to_other_wu() {
    let (actor, other, mut tx) = boot(31);
    tx.state_mut().group_overrides.insert(actor, "wu".into());
    tx.state_mut().group_overrides.insert(other, "wu".into());
    clear_hand(&mut tx, actor);
    clear_hand(&mut tx, other);
    let guguoanbang = inject_guguoanbang(&mut tx, actor);
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
                card: guguoanbang,
                targets: vec![actor],
                card_targets: Vec::new(),
            },
        },
    );
    assert!(
        matches!(output.result, CommandResult::Accepted { .. }),
        "guguoanbang play rejected: {:?}",
        output.result
    );
    let mut key = 2;
    while state.has_pending_choice()
        && state.resolution.continuation_kind == ContinuationKind::WuxieWindow
    {
        state = resolve(state, vec![], true, key);
        key += 1;
    }
    assert!(state.has_pending_choice(), "discard choice missing");
    assert_eq!(state.hand_count(actor), 8, "drew eight");
    let hand: Vec<u32> = state.hand_cards(actor).iter().map(|c| c.raw()).collect();
    let discard6 = hand.iter().take(6).copied().collect::<Vec<_>>();
    state = resolve(state, discard6.clone(), false, key);
    key += 1;
    // 吴势力 → 给牌选择：从弃置的 6 张中选 1–2 张。
    assert!(state.has_pending_choice(), "give-cards choice missing");
    let give2 = discard6.iter().take(2).copied().collect();
    state = resolve(state, give2, false, key);
    key += 1;
    // 选择其他吴目标。
    assert!(state.has_pending_choice(), "give-target choice missing");
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::Custom
    );
    state = resolve(state, vec![other.raw()], false, key);
    assert!(!state.has_pending_choice(), "settled");
    assert_eq!(state.hand_count(other), 2, "recipient gained two cards");
}
