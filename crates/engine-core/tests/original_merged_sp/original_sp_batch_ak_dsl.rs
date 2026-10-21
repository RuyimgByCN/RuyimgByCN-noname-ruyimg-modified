//! Golden for sp batch-AK DSL rewrites:
//! 疾刺·亡 (rejici2) death: the killer discards `7 - owner's regushe marks`
//!   hand-or-equip cards, then loses 1 HP (new expression-count discard
//!   primitive `discard_hand_equip_expr`, reusing the existing VM op);
//! 雍州 (pxyongzhou) on-gain optional: draw 2, then discard hand down to the
//!   hand limit (upgrade from the previous fixed-discard-1 approximation to
//!   exact semantics via `discard_hand_expr`).

use engine_api::{
    CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey,
    MarkDefId, PlayerId, SkillDefId,
};
use engine_core::command::apply_command;
use engine_core::effect::run_effects;
use engine_core::query::{query, QueryContext};
use engine_core::state::{Card, CardKind, ContinuationKind, GameState, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use engine_core::Engine;
use rule_api::{EffectRequest, QueryRequest, QueryResponse};

const REJICI2: u32 = 3_976_772_344;
const REGUSHE: u32 = 3_913_395_307;
const PXYONGZHOU: u32 = 2_600_588_951;
const PACKAGE: &[u8] = include_bytes!("../../../../content/original/ported/sp/sp.npack.json");

fn started(seed: u64) -> (GameState, PlayerId) {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
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
    let state = engine.state().clone();
    let owner = state.turn.current;
    (state, owner)
}

fn other_of(state: &GameState, owner: PlayerId) -> PlayerId {
    state
        .players
        .iter_live()
        .find(|player| player.id != owner)
        .unwrap()
        .id
}

fn resolve(
    state: GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    cancelled: bool,
    key: u64,
) -> (GameState, engine_core::command::PipelineResult) {
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

fn inject_hand(state: &mut GameState, player: PlayerId, kind: CardKind, tag: u32) -> CardId {
    let hand = state.zones.find(ZoneKind::Hand, player).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: engine_api::CardDefId::new(91_800 + tag),
        kind,
        suit: engine_api::Suit::Club,
        rank: 5,
        weapon_range: 0,
        zone: hand,
        owner: player,
        face_up: true,
    });
    state.zones.get_mut(hand).unwrap().cards.push(card);
    card
}

fn hand_limit(state: &GameState, player: PlayerId) -> i32 {
    match query(
        state,
        &QueryRequest::HandLimit { player },
        &mut QueryContext::new(),
    ) {
        QueryResponse::Int(value) => value,
        other => panic!("unexpected hand-limit response: {other:?}"),
    }
}

#[test]
fn package_publishes_batch_ak_contract() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    // The expression-count discard reuses the existing HostEffect op: no ABI bump.
    assert_eq!(package.manifest.rule_vm_abi_version, 745);
    let mut seen = std::collections::BTreeSet::new();
    for trigger in &package.trigger_index {
        if matches!(trigger.skill_def_id, REJICI2 | PXYONGZHOU) {
            seen.insert((
                trigger.skill_def_id,
                trigger.event.as_str(),
                trigger.filter.as_str(),
                trigger.optional,
            ));
        }
    }
    assert!(seen.contains(&(REJICI2, "death", "owner_is_target_source_alive", false)));
    assert!(seen.contains(&(
        PXYONGZHOU,
        "skill_added",
        "owner_skill_added_self_2600588951",
        true
    )));
}

#[test]
fn rejici2_killer_discards_seven_minus_marks_then_loses_hp() {
    let (state, owner) = started(2_026_081_530);
    let other = other_of(&state, owner);

    let mut tx = Transaction::begin(state);
    // Strip pre-existing skills so death triggers stay deterministic.
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(REJICI2),
    });
    // Three regushe marks on the dying owner → the killer discards 4 cards.
    effects.push(EffectRequest::AddMark {
        player: owner,
        mark: MarkDefId::new(REGUSHE),
        delta: 3,
    });
    run_effects(&mut tx, effects).unwrap();
    let mut state = tx.commit().state;

    // Give the killer exactly three hand cards so the discard count is observable.
    inject_hand(&mut state, other, CardKind::Tao, 1);
    inject_hand(&mut state, other, CardKind::Shan, 2);
    inject_hand(&mut state, other, CardKind::Jiu, 3);
    let other_hand = state.hand_count(other);
    inject_hand(&mut state, other, CardKind::Sha, 4);
    let hp_before = state.players.get(other).unwrap().hp;

    let mut tx = Transaction::begin(state);
    let effects = dispatch_for_events(
        &mut tx,
        &[engine_api::DomainEvent::PlayerKilled {
            player: owner,
            killer: other,
            hand_equip_cards: vec![],
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    run_effects(&mut tx, effects).unwrap();
    let state = tx.commit().state;
    // 7 - 3 marks = 4 discarded from the killer's hand-or-equip.
    assert_eq!(state.hand_count(other), other_hand + 1 - 4);
    assert_eq!(state.players.get(other).unwrap().hp, hp_before - 1);
}

#[test]
fn rejici2_full_marks_skip_discard_but_keep_lose_hp() {
    let (state, owner) = started(2_026_081_531);
    let other = other_of(&state, owner);

    let mut tx = Transaction::begin(state);
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(REJICI2),
    });
    // Seven regushe marks → discard count 7 - 7 = 0, only the HP loss remains.
    effects.push(EffectRequest::AddMark {
        player: owner,
        mark: MarkDefId::new(REGUSHE),
        delta: 7,
    });
    run_effects(&mut tx, effects).unwrap();
    let state = tx.commit().state;
    let other_hand = state.hand_count(other);
    let hp_before = state.players.get(other).unwrap().hp;

    let mut tx = Transaction::begin(state);
    let effects = dispatch_for_events(
        &mut tx,
        &[engine_api::DomainEvent::PlayerKilled {
            player: owner,
            killer: other,
            hand_equip_cards: vec![],
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    run_effects(&mut tx, effects).unwrap();
    let state = tx.commit().state;
    assert_eq!(state.hand_count(other), other_hand);
    assert_eq!(state.players.get(other).unwrap().hp, hp_before - 1);
}

#[test]
fn rejici2_no_killer_does_not_fire() {
    let (state, owner) = started(2_026_081_532);
    let other = other_of(&state, owner);

    let mut tx = Transaction::begin(state);
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(REJICI2),
    });
    run_effects(&mut tx, effects).unwrap();
    let state = tx.commit().state;
    let other_hand = state.hand_count(other);
    let hp_before = state.players.get(other).unwrap().hp;

    let mut tx = Transaction::begin(state);
    let effects = dispatch_for_events(
        &mut tx,
        &[engine_api::DomainEvent::PlayerKilled {
            player: owner,
            killer: PlayerId::NONE,
            hand_equip_cards: vec![],
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    assert!(effects.is_empty());
    let state = tx.commit().state;
    assert_eq!(state.hand_count(other), other_hand);
    assert_eq!(state.players.get(other).unwrap().hp, hp_before);
    let _ = other;
}

#[test]
fn pxyongzhou_draws_two_then_discards_down_to_limit() {
    let (state, owner) = started(2_026_081_533);
    let hand_before = state.hand_count(owner) as i32;
    let limit = hand_limit(&state, owner);

    let mut tx = Transaction::begin(state);
    let effects = run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(PXYONGZHOU),
        }],
    );
    assert!(matches!(
        effects,
        Err(engine_core::effect::EffectError::ChoiceRequested)
    ));
    assert_eq!(
        tx.state().resolution.continuation_kind,
        ContinuationKind::OptionalTrigger
    );
    let tx = tx.commit();
    let (state, result) = resolve(tx.state, owner, vec![1], false, 1);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    // Draw 2, then discard the excess over the hand limit.
    let expected = (hand_before + 2).min(limit) as u32;
    assert_eq!(state.hand_count(owner), expected);
}

#[test]
fn pxyongzhou_cancel_keeps_hand_plus_nothing() {
    let (state, owner) = started(2_026_081_534);
    let hand_before = state.hand_count(owner);

    let mut tx = Transaction::begin(state);
    let effects = run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(PXYONGZHOU),
        }],
    );
    assert!(matches!(
        effects,
        Err(engine_core::effect::EffectError::ChoiceRequested)
    ));
    let tx = tx.commit();
    let (state, result) = resolve(tx.state, owner, vec![], true, 2);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(owner), hand_before);
}
