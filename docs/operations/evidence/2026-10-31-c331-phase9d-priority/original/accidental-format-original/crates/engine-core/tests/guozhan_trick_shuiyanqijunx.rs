//! Guozhan 水淹七军 (shuiyanqijunx) trick settlement — ADR 0615.
//!
//! 水淹七军（国战）：对一名装备区里有牌的其他角色使用。目标选择一项：
//! ⒈弃置装备区里的所有牌；⒉受到使用者造成的 1 点雷电伤害。
//! 无装备可弃时直接受 1 点雷电伤害。

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

fn inject(tx: &mut Transaction, owner: PlayerId, kind: CardKind, offset: u32) -> CardId {
    let zone = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(462_000 + offset),
        kind,
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
fn shuiyanqijunx_discard_all_equipment_branch() {
    let (actor, other, mut tx) = boot(99);
    let clear: Vec<_> = tx.state().hand_cards(actor);
    if !clear.is_empty() {
        run_effects(
            &mut tx,
            vec![EffectRequest::DiscardCards {
                player: actor,
                cards: clear,
            }],
        )
        .unwrap();
    }
    let clear2: Vec<_> = tx.state().hand_cards(other);
    if !clear2.is_empty() {
        run_effects(
            &mut tx,
            vec![EffectRequest::DiscardCards {
                player: other,
                cards: clear2,
            }],
        )
        .unwrap();
    }
    // 给目标一件武器装备 + 一张手牌（防具区留一件：弃全部装备分支需要 ≥1 件可弃装备）。
    let zone = tx.state().zones.find(ZoneKind::Equip, other).unwrap();
    let weapon = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(100), // 雌雄双股剑 def 家族，装备
        kind: CardKind::Weapon,
        suit: Suit::Spade,
        rank: 5,
        weapon_range: 2,
        zone,
        owner: other,
        face_up: true,
    });
    tx.state_mut()
        .zones
        .get_mut(zone)
        .unwrap()
        .cards
        .push(weapon);
    let shuiyan = inject(&mut tx, actor, CardKind::TrickShuiyanqijunx, 0);
    tx.state_mut().turn.current = actor;
    tx.state_mut().turn.phase = Phase::Play;
    let state = tx.commit().state;
    assert_eq!(
        state
            .zones
            .get(state.zones.find(ZoneKind::Equip, other).unwrap())
            .unwrap()
            .cards
            .len(),
        1
    );

    // 使用水淹七军（目标 = other）。
    let initial_hp = state.players.get(other).unwrap().hp;
    let (mut state, output) = apply_command(
        state.clone(),
        Command {
            actor,
            expected_revision: ExpectedRevision(state.revision),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::PlayCard {
                card: shuiyan,
                targets: vec![other],
                card_targets: Vec::new(),
            },
        },
    );
    assert!(
        matches!(output.result, CommandResult::Accepted { .. }),
        "shuiyanqijunx play rejected: {:?}",
        output.result
    );
    let mut key = 2;
    while state.has_pending_choice()
        && state.resolution.continuation_kind == ContinuationKind::WuxieWindow
    {
        state = resolve(state, vec![], true, key);
        key += 1;
    }

    // 目标选择「弃置全部装备牌」（选项 1）。
    assert!(
        state.has_pending_choice(),
        "shuiyanqijunx target choice missing"
    );
    assert_eq!(state.resolution.pending_observer, other.raw());
    state = resolve(state, vec![1], false, key);
    assert!(!state.has_pending_choice(), "shuiyanqijunx settled");
    assert_eq!(
        state
            .zones
            .get(state.zones.find(ZoneKind::Equip, other).unwrap())
            .unwrap()
            .cards
            .len(),
        0,
        "all equipment discarded"
    );
    let hp = state.players.get(other).unwrap().hp;
    assert_eq!(hp, initial_hp, "no damage on discard branch");
}

#[test]
fn shuiyanqijunx_requires_an_equipment_target() {
    // 上游 filterTarget（guozhan）：目标须为装备区有牌的其他角色——无装备目标不可选。
    // （「无装备可弃 → 直接受雷伤」分支仅在装备被保护不可弃时于结算内触发。）
    let (actor, other, mut tx) = boot(7);
    let clear: Vec<_> = tx.state().hand_cards(actor);
    if !clear.is_empty() {
        run_effects(
            &mut tx,
            vec![EffectRequest::DiscardCards {
                player: actor,
                cards: clear,
            }],
        )
        .unwrap();
    }
    let clear2: Vec<_> = tx.state().hand_cards(other);
    if !clear2.is_empty() {
        run_effects(
            &mut tx,
            vec![EffectRequest::DiscardCards {
                player: other,
                cards: clear2,
            }],
        )
        .unwrap();
    }
    let shuiyan = inject(&mut tx, actor, CardKind::TrickShuiyanqijunx, 1);
    tx.state_mut().turn.current = actor;
    tx.state_mut().turn.phase = Phase::Play;
    let state = tx.commit().state;
    let (_, output) = apply_command(
        state.clone(),
        Command {
            actor,
            expected_revision: ExpectedRevision(state.revision),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::PlayCard {
                card: shuiyan,
                targets: vec![other],
                card_targets: Vec::new(),
            },
        },
    );
    assert!(
        matches!(output.result, CommandResult::Rejected { .. }),
        "no-equipment target must be rejected"
    );
}

#[test]
fn shuiyanqijunx_take_thunder_damage_branch() {
    let (actor, other, mut tx) = boot(31);
    let clear: Vec<_> = tx.state().hand_cards(actor);
    if !clear.is_empty() {
        run_effects(
            &mut tx,
            vec![EffectRequest::DiscardCards {
                player: actor,
                cards: clear,
            }],
        )
        .unwrap();
    }
    let clear2: Vec<_> = tx.state().hand_cards(other);
    if !clear2.is_empty() {
        run_effects(
            &mut tx,
            vec![EffectRequest::DiscardCards {
                player: other,
                cards: clear2,
            }],
        )
        .unwrap();
    }
    let equip_zone = tx.state().zones.find(ZoneKind::Equip, other).unwrap();
    let weapon = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(101), // 诸葛连弩
        kind: CardKind::Weapon,
        suit: Suit::Heart,
        rank: 1,
        weapon_range: 1,
        zone: equip_zone,
        owner: other,
        face_up: true,
    });
    tx.state_mut()
        .zones
        .get_mut(equip_zone)
        .unwrap()
        .cards
        .push(weapon);
    let shuiyan = inject(&mut tx, actor, CardKind::TrickShuiyanqijunx, 2);
    tx.state_mut().turn.current = actor;
    tx.state_mut().turn.phase = Phase::Play;
    let state = tx.commit().state;
    let initial_hp = state.players.get(other).unwrap().hp;
    let (mut state, output) = apply_command(
        state.clone(),
        Command {
            actor,
            expected_revision: ExpectedRevision(state.revision),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::PlayCard {
                card: shuiyan,
                targets: vec![other],
                card_targets: Vec::new(),
            },
        },
    );
    assert!(
        matches!(output.result, CommandResult::Accepted { .. }),
        "shuiyanqijunx play rejected: {:?}",
        output.result
    );
    let mut key = 2;
    while state.has_pending_choice()
        && state.resolution.continuation_kind == ContinuationKind::WuxieWindow
    {
        state = resolve(state, vec![], true, key);
        key += 1;
    }
    // 目标选择「受 1 点雷电伤害」（选项 2）。
    assert!(state.has_pending_choice(), "target choice missing");
    state = resolve(state, vec![2], false, key);
    assert!(!state.has_pending_choice(), "shuiyanqijunx settled");
    assert_eq!(
        state.players.get(other).unwrap().hp,
        initial_hp - 1,
        "target took 1 thunder damage"
    );
    assert_eq!(
        state
            .zones
            .get(state.zones.find(ZoneKind::Equip, other).unwrap())
            .unwrap()
            .cards
            .len(),
        1,
        "equipment kept on damage branch"
    );
}
