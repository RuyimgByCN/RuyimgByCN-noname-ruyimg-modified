//! Golden for SP「甚贤」and OLD「旧甚贤」discard-cause semantics.

use engine_api::{
    ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey, PlayerId,
    SkillDefId,
};
use engine_core::command::apply_command;
use engine_core::effect::{run_effects, EffectError};
use engine_core::state::{CardKind, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::EffectRequest;

const SHENXIAN: u32 = 3_316_671_408;
const OLD_SHENXIAN: u32 = 2_584_430_535;
const SP_PACKAGE: &[u8] = include_bytes!("../../../../content/original/ported/sp/sp.npack.json");
const OLD_PACKAGE: &[u8] = include_bytes!("../../../../content/original/ported/old/old.npack.json");

fn isolated(package_bytes: &[u8], skill_def: u32) -> (Transaction, PlayerId, PlayerId) {
    let package = content_schema::ContentPackage::from_json_bytes(package_bytes).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(skill_def as u64),
            kind: CommandKind::StartGame {
                player_count: 3,
                rng_seed: skill_def as u64,
                ruleset: "identity-standard".into(),
            },
        }),
        CommandResult::Accepted { .. }
    ));
    let players = engine.state().players.alive_by_seat();
    let (owner, discarder) = (players[0], players[1]);
    let mut tx = Transaction::begin(engine.state().clone());
    let mut setup: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    setup.extend([
        EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(skill_def),
        },
        EffectRequest::SetTurnState {
            current: discarder,
            phase: Phase::Play.to_u8(),
        },
    ]);
    run_effects(&mut tx, setup).unwrap();
    (tx, owner, discarder)
}

fn accept_optional(tx: Transaction, owner: PlayerId, key: u64) -> engine_core::state::GameState {
    let choice_id = ChoiceId::new(tx.state().resolution.pending_choice_id);
    let (state, pipeline) = apply_command(
        tx.state().clone(),
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(tx.state().revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections: vec![1],
                cancelled: false,
            },
        },
    );
    assert!(matches!(pipeline.result, CommandResult::Accepted { .. }));
    state
}

fn set_kind(
    tx: &mut Transaction,
    player: PlayerId,
    index: usize,
    kind: CardKind,
) -> engine_api::CardId {
    let card = tx.state().hand_cards(player)[index];
    tx.state_mut().cards.get_mut(card).unwrap().kind = kind;
    card
}

#[test]
fn package_filters_preserve_the_new_and_old_turn_limit_difference() {
    let sp = content_schema::ContentPackage::from_json_bytes(SP_PACKAGE).unwrap();
    assert!(sp.trigger_index.iter().any(|trigger| {
        trigger.skill_def_id == SHENXIAN
            && trigger.event == "cards_moved"
            && trigger.filter == "owner_alive_other_discarded_basic_outside_turn_once"
            && trigger.optional
    }));
    let old = content_schema::ContentPackage::from_json_bytes(OLD_PACKAGE).unwrap();
    assert!(old.trigger_index.iter().any(|trigger| {
        trigger.skill_def_id == OLD_SHENXIAN
            && trigger.event == "cards_moved"
            && trigger.filter == "owner_alive_other_discarded_basic_outside_turn"
            && trigger.optional
    }));
}

#[test]
fn shenxian_requires_an_actual_basic_discard_outside_the_owner_turn() {
    let (mut tx, owner, discarder) = isolated(SP_PACKAGE, SHENXIAN);
    let before = tx.state().hand_count(owner);
    let basic_move = set_kind(&mut tx, discarder, 0, CardKind::Shan);
    let non_basic = set_kind(&mut tx, discarder, 1, CardKind::TrickWuzhong);
    let basic_discard = set_kind(&mut tx, discarder, 2, CardKind::Sha);
    let hand = tx.state().zones.find(ZoneKind::Hand, discarder).unwrap();
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();

    // A basic card reaching the discard pile through an unspecified use/response-style move is
    // not a discard and must not offer 甚贤.
    run_effects(
        &mut tx,
        vec![EffectRequest::MoveCards {
            cards: vec![basic_move],
            to: discard,
            from: Some(hand),
        }],
    )
    .unwrap();
    assert!(!tx.state().has_pending_choice());

    run_effects(
        &mut tx,
        vec![EffectRequest::DiscardCards {
            player: discarder,
            cards: vec![non_basic],
        }],
    )
    .unwrap();
    assert!(!tx.state().has_pending_choice());

    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::DiscardCards {
                player: discarder,
                cards: vec![basic_discard],
            }],
        )
        .unwrap_err(),
        EffectError::ChoiceRequested
    );
    let state = accept_optional(tx, owner, 1);
    assert_eq!(state.hand_count(owner), before + 1);

    let mut tx = Transaction::begin(state);
    run_effects(
        &mut tx,
        vec![EffectRequest::DrawCards {
            player: discarder,
            count: 1,
        }],
    )
    .unwrap();
    let own_turn_basic = set_kind(&mut tx, discarder, 0, CardKind::Tao);
    run_effects(
        &mut tx,
        vec![
            EffectRequest::SetTurnState {
                current: owner,
                phase: Phase::Play.to_u8(),
            },
            EffectRequest::DiscardCards {
                player: discarder,
                cards: vec![own_turn_basic],
            },
        ],
    )
    .unwrap();
    assert!(!tx.state().has_pending_choice());
    assert_eq!(tx.state().hand_count(owner), before + 1);
}

#[test]
fn shenxian_is_once_per_character_turn_but_oldshenxian_is_not() {
    let (mut tx, owner, discarder) = isolated(SP_PACKAGE, SHENXIAN);
    let first = set_kind(&mut tx, discarder, 0, CardKind::Sha);
    let second = set_kind(&mut tx, discarder, 1, CardKind::Tao);
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::DiscardCards {
                player: discarder,
                cards: vec![first],
            }],
        )
        .unwrap_err(),
        EffectError::ChoiceRequested
    );
    let state = accept_optional(tx, owner, 2);
    let after_first = state.hand_count(owner);
    let mut tx = Transaction::begin(state);
    run_effects(
        &mut tx,
        vec![EffectRequest::DiscardCards {
            player: discarder,
            cards: vec![second],
        }],
    )
    .unwrap();
    assert!(!tx.state().has_pending_choice());
    assert_eq!(tx.state().hand_count(owner), after_first);

    let (mut tx, owner, discarder) = isolated(OLD_PACKAGE, OLD_SHENXIAN);
    let first = set_kind(&mut tx, discarder, 0, CardKind::Sha);
    let second = set_kind(&mut tx, discarder, 1, CardKind::Tao);
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::DiscardCards {
                player: discarder,
                cards: vec![first],
            }],
        )
        .unwrap_err(),
        EffectError::ChoiceRequested
    );
    let state = accept_optional(tx, owner, 3);
    let after_first = state.hand_count(owner);
    let mut tx = Transaction::begin(state);
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::DiscardCards {
                player: discarder,
                cards: vec![second],
            }],
        )
        .unwrap_err(),
        EffectError::ChoiceRequested
    );
    let state = accept_optional(tx, owner, 4);
    assert_eq!(state.hand_count(owner), after_first + 1);
}
