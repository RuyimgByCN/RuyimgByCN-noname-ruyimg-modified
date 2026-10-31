//! Guozhan 联军盛宴 (lianjunshengyan) trick settlement — ADR 0620.
//!
//! 联军盛宴（国战）：对使用者自己及一敌方势力全体使用。非己目标摸一张并重置
//! （解除横置）；自己按敌方势力人数 X 分配「摸 Y 张 + 回 X−Y 点」（Y∈[0,X]，回复
//! 以缺体为上限）。

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, MarkDefId, PlayerId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::run_effects;
use engine_core::state::{Card, CardKind, ContinuationKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::EffectRequest;

fn inject_lianjunshengyan(tx: &mut Transaction, owner: PlayerId) -> CardId {
    let zone = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(467_000),
        kind: CardKind::TrickLianjunshengyan,
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

fn clear_hand(tx: &mut Transaction, player: PlayerId) {
    let cards = tx.state().hand_cards(player);
    if !cards.is_empty() {
        run_effects(tx, vec![EffectRequest::DiscardCards { player, cards }]).unwrap();
    }
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

#[test]
fn lianjunshengyan_self_draws_and_enemy_draws_resets() {
    let (actor, other, mut tx) = boot(99);
    tx.state_mut().group_overrides.insert(actor, "wei".into());
    tx.state_mut().group_overrides.insert(other, "shu".into());
    clear_hand(&mut tx, actor);
    clear_hand(&mut tx, other);
    // 预置敌方已横置 → 重置分支。
    run_effects(
        &mut tx,
        vec![EffectRequest::AddMark {
            player: other,
            mark: MarkDefId::new(engine_core::content::MARK_CHAIN),
            delta: 1,
        }],
    )
    .unwrap();
    let lianjunshengyan = inject_lianjunshengyan(&mut tx, actor);
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
                card: lianjunshengyan,
                targets: vec![other],
                card_targets: Vec::new(),
            },
        },
    );
    assert!(
        matches!(output.result, CommandResult::Accepted { .. }),
        "lianjunshengyan play rejected: {:?}",
        output.result
    );
    let mut key = 2;
    while state.has_pending_choice()
        && state.resolution.continuation_kind == ContinuationKind::WuxieWindow
    {
        state = resolve(state, vec![], true, key);
        key += 1;
    }
    // 自己满血 → 摸 X=1，无 choice；敌方摸 1 并重置。
    assert!(!state.has_pending_choice(), "full-hp self settled");
    assert_eq!(state.hand_count(actor), 1, "self drew X = 1");
    assert_eq!(state.hand_count(other), 1, "enemy drew one");
    assert!(
        !state
            .players
            .get(other)
            .unwrap()
            .marks
            .iter()
            .any(|(m, c)| *m == engine_core::content::MARK_CHAIN && *c > 0),
        "enemy reset (unlinked)"
    );
}

#[test]
fn lianjunshengyan_damaged_self_chooses_draw_recover_split() {
    let (actor, other, mut tx) = boot(31);
    tx.state_mut().group_overrides.insert(actor, "wei".into());
    tx.state_mut().group_overrides.insert(other, "shu".into());
    clear_hand(&mut tx, actor);
    clear_hand(&mut tx, other);
    // 使自己受伤 1 点。
    run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source: PlayerId::NONE,
            target: actor,
            amount: 1,
            is_lose_hp: false,
            nature: 0,
        }],
    )
    .unwrap();
    let hp_before = tx.state().players.get(actor).unwrap().hp;
    let lianjunshengyan = inject_lianjunshengyan(&mut tx, actor);
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
                card: lianjunshengyan,
                targets: vec![other],
                card_targets: Vec::new(),
            },
        },
    );
    assert!(
        matches!(output.result, CommandResult::Accepted { .. }),
        "lianjunshengyan play rejected: {:?}",
        output.result
    );
    let mut key = 2;
    while state.has_pending_choice()
        && state.resolution.continuation_kind == ContinuationKind::WuxieWindow
    {
        state = resolve(state, vec![], true, key);
        key += 1;
    }
    // 自己受伤 → 摸/回分配 choice（X=1，cap=1 → 选项 [0, 1]）。
    assert!(
        state.has_pending_choice(),
        "draw/recover split choice missing"
    );
    assert_eq!(state.resolution.pending_observer, actor.raw());
    // 选「回 1 摸 0」。
    state = resolve(state, vec![1], false, key);
    // 敌方目标随后结算：摸 1。
    assert!(
        !state.has_pending_choice(),
        "enemy draw settles without a choice"
    );
    assert_eq!(
        state.players.get(actor).unwrap().hp,
        hp_before + 1,
        "self recovered one"
    );
    assert_eq!(state.hand_count(actor), 0, "self drew zero (chose recover)");
    assert_eq!(state.hand_count(other), 1, "enemy drew one");
}
