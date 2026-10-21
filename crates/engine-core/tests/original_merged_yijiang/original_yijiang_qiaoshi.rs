//! Golden for 一将成名夏侯氏「樵拾」的等手牌结束阶段语义。

use engine_api::{
    ChoiceId, Command, CommandKind, CommandResult, DomainEvent, ExpectedRevision, IdempotencyKey,
    PlayerId, SkillDefId, TurnPhaseTag,
};
use engine_core::command::apply_command;
use engine_core::effect::{run_effects, EffectError};
use engine_core::state::CardKind;
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use engine_core::Engine;
use rule_api::EffectRequest;

const QIAOSHI: u32 = 2_607_312_044;
const JYZONGSHI_OLD: u32 = 2_968_553_132;
const PACKAGE: &[u8] =
    include_bytes!("../../../../content/original/ported/yijiang/yijiang.npack.json");

fn qiaoshi_event(player: PlayerId) -> DomainEvent {
    DomainEvent::TurnStateChanged {
        current: player,
        phase: TurnPhaseTag::End,
    }
}

fn isolated_state() -> (Transaction, PlayerId, PlayerId) {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == QIAOSHI && !skill.locked));

    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    engine.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(0),
        kind: CommandKind::StartGame {
            player_count: 2,
            rng_seed: 20_260_714,
            ruleset: "identity-standard".into(),
        },
    });
    let owner = engine.state().turn.current;
    let other = engine
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .find(|player| *player != owner)
        .unwrap();
    let mut tx = Transaction::begin(engine.state().clone());
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(QIAOSHI),
    });
    run_effects(&mut tx, effects).unwrap();
    let owner_hand = tx.state().hand_count(owner);
    let other_hand = tx.state().hand_count(other);
    if owner_hand > other_hand {
        run_effects(
            &mut tx,
            vec![EffectRequest::DiscardFromHand {
                player: owner,
                count: owner_hand - other_hand,
            }],
        )
        .unwrap();
    } else if other_hand > owner_hand {
        run_effects(
            &mut tx,
            vec![EffectRequest::DrawCards {
                player: owner,
                count: other_hand - owner_hand,
            }],
        )
        .unwrap();
    }
    (tx, owner, other)
}

#[test]
fn equal_hands_offer_owner_choice_and_accepting_draws_for_both() {
    let (mut tx, owner, other) = isolated_state();
    assert_eq!(tx.state().hand_count(owner), tx.state().hand_count(other));
    let owner_hand = tx.state().hand_count(owner);
    let other_hand = tx.state().hand_count(other);

    let paused = dispatch_for_events(
        &mut tx,
        &[qiaoshi_event(other)],
        &mut TriggerFireRecord::new(),
    )
    .unwrap_err();
    assert_eq!(paused, EffectError::ChoiceRequested);

    let (state, result) = apply_command(
        tx.state().clone(),
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::ResolveChoice {
                choice_id: ChoiceId::new(tx.state().resolution.pending_choice_id),
                selections: vec![1],
                cancelled: false,
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(other), other_hand + 1);
    assert_eq!(state.hand_count(owner), owner_hand + 1);
}

#[test]
fn unequal_hands_or_owners_own_end_phase_do_not_offer_a_choice() {
    let (mut tx, owner, other) = isolated_state();
    run_effects(
        &mut tx,
        vec![EffectRequest::DiscardFromHand {
            player: owner,
            count: 1,
        }],
    )
    .unwrap();
    assert_ne!(tx.state().hand_count(owner), tx.state().hand_count(other));

    let effects = dispatch_for_events(
        &mut tx,
        &[qiaoshi_event(other)],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    assert!(effects.is_empty());
    assert!(!tx.state().has_pending_choice());

    run_effects(
        &mut tx,
        vec![EffectRequest::DrawCards {
            player: owner,
            count: 1,
        }],
    )
    .unwrap();
    assert_eq!(tx.state().hand_count(owner), tx.state().hand_count(other));
    let effects = dispatch_for_events(
        &mut tx,
        &[qiaoshi_event(owner)],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    assert!(effects.is_empty());
    assert!(!tx.state().has_pending_choice());
}

#[test]
fn old_zongshi_only_draws_for_another_players_single_target_trick() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    engine.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(30),
        kind: CommandKind::StartGame {
            player_count: 2,
            rng_seed: 20_260_714,
            ruleset: "identity-standard".into(),
        },
    });
    let owner = engine.state().turn.current;
    let other = engine
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .find(|player| *player != owner)
        .unwrap();
    let mut tx = Transaction::begin(engine.state().clone());
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(JYZONGSHI_OLD),
    });
    run_effects(&mut tx, effects).unwrap();
    let card = tx.state().hand_cards(other)[0];
    tx.state_mut().cards.get_mut(card).unwrap().kind = CardKind::TrickGuohe;
    let hand = tx.state().hand_count(owner);

    let paused = dispatch_for_events(
        &mut tx,
        &[DomainEvent::CardUsed {
            player: other,
            card,
            targets: vec![owner],
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap_err();
    assert_eq!(paused, EffectError::ChoiceRequested);
    let (state, result) = apply_command(
        tx.state().clone(),
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(31),
            kind: CommandKind::ResolveChoice {
                choice_id: ChoiceId::new(tx.state().resolution.pending_choice_id),
                selections: vec![1],
                cancelled: false,
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(owner), hand + 1);

    let mut tx = Transaction::begin(state);
    for event in [
        DomainEvent::CardUsed {
            player: other,
            card,
            targets: vec![owner, other],
        },
        DomainEvent::CardUsed {
            player: owner,
            card,
            targets: vec![owner],
        },
    ] {
        dispatch_for_events(&mut tx, &[event], &mut TriggerFireRecord::new()).unwrap();
        assert!(!tx.state().has_pending_choice());
    }
}
