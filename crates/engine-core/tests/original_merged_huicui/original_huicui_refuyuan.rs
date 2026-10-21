//! Golden for HUICUI 陈登「扶援」and bounded per-target red-card turn history.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{run_effects, EffectError};
use engine_core::state::{Card, CardKind, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use engine_core::Engine;
use rule_api::EffectRequest;

const REFUYUAN: u32 = 2_444_177_428;
const HUICUI: &[u8] =
    include_bytes!("../../../../content/original/ported/huicui/huicui.npack.json");
const SHIJI: &[u8] = include_bytes!("../../../../content/original/ported/shiji/shiji.npack.json");

fn loaded() -> (Transaction, PlayerId, PlayerId, PlayerId) {
    let huicui = content_schema::ContentPackage::from_json_bytes(HUICUI).unwrap();
    let shiji = content_schema::ContentPackage::from_json_bytes(SHIJI).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&shiji).unwrap();
    engine.load_package(&huicui).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(REFUYUAN as u64),
            kind: CommandKind::StartGame {
                player_count: 3,
                rng_seed: 20_260_718,
                ruleset: "identity-standard".into(),
            },
        }),
        CommandResult::Accepted { .. }
    ));
    let seats = engine.state().players.alive_by_seat();
    let (owner, target, source) = (seats[0], seats[1], seats[2]);
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
        skill_def: SkillDefId::new(REFUYUAN),
    });
    run_effects(&mut tx, effects).unwrap();
    (tx, owner, target, source)
}

fn inject(tx: &mut Transaction, owner: PlayerId, kind: CardKind, suit: Suit) -> CardId {
    let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(90_000 + kind.tag() as u32),
        kind,
        suit,
        rank: 7,
        weapon_range: 0,
        zone: hand,
        owner,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(hand).unwrap().cards.push(card);
    card
}

fn dispatch(
    tx: &mut Transaction,
    user: PlayerId,
    card: CardId,
    targets: Vec<PlayerId>,
) -> Result<Vec<EffectRequest>, EffectError> {
    dispatch_for_events(
        tx,
        &[DomainEvent::CardUsed {
            player: user,
            card,
            targets,
        }],
        &mut TriggerFireRecord::new(),
    )
}

fn accept(tx: Transaction, owner: PlayerId, key: u64) -> engine_core::state::GameState {
    let (state, result) = apply_command(
        tx.state().clone(),
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(tx.state().revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id: ChoiceId::new(tx.state().resolution.pending_choice_id),
                selections: vec![1],
                cancelled: false,
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    state
}

#[test]
fn committed_package_declares_refuyuan_and_abi29() {
    let package = content_schema::ContentPackage::from_json_bytes(HUICUI).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 487);
    let trigger = package
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == REFUYUAN)
        .unwrap();
    assert_eq!(trigger.event, "card_targeted");
    assert_eq!(
        trigger.filter,
        "owner_alive_sha_target_no_other_red_card_this_turn"
    );
    assert!(trigger.optional);
}

#[test]
fn first_sha_target_is_offered_and_draws_for_event_player() {
    let (mut tx, owner, target, source) = loaded();
    let unrelated_target = owner;
    let prior_red = inject(&mut tx, source, CardKind::Other, Suit::Heart);
    assert!(dispatch(&mut tx, source, prior_red, vec![unrelated_target])
        .unwrap()
        .is_empty());
    let sha = inject(&mut tx, source, CardKind::Sha, Suit::Spade);
    let before = tx.state().hand_count(target);
    assert_eq!(
        dispatch(&mut tx, source, sha, vec![target]).unwrap_err(),
        EffectError::ChoiceRequested
    );
    let state = accept(tx, owner, 1);
    assert_eq!(state.hand_count(target), before + 1);
}

#[test]
fn earlier_red_card_from_any_source_blocks_later_sha_for_same_target() {
    let (mut tx, _owner, target, source) = loaded();
    let prior_red = inject(&mut tx, source, CardKind::Other, Suit::Diamond);
    assert!(dispatch(&mut tx, source, prior_red, vec![target])
        .unwrap()
        .is_empty());
    let sha = inject(&mut tx, source, CardKind::Sha, Suit::Spade);
    assert!(dispatch(&mut tx, source, sha, vec![target])
        .unwrap()
        .is_empty());
    assert!(!tx.state().has_pending_choice());
}

#[test]
fn later_red_card_in_same_batch_does_not_cancel_earlier_sha_offer() {
    let (mut tx, _owner, target, source) = loaded();
    let sha = inject(&mut tx, source, CardKind::Sha, Suit::Spade);
    let later_red = inject(&mut tx, source, CardKind::Other, Suit::Heart);
    assert_eq!(
        dispatch_for_events(
            &mut tx,
            &[
                DomainEvent::CardUsed {
                    player: source,
                    card: sha,
                    targets: vec![target],
                },
                DomainEvent::CardUsed {
                    player: source,
                    card: later_red,
                    targets: vec![target],
                },
            ],
            &mut TriggerFireRecord::new(),
        )
        .unwrap_err(),
        EffectError::ChoiceRequested
    );
}

#[test]
fn same_physical_red_card_is_excluded_and_turn_boundary_clears_history() {
    let (mut tx, owner, target, source) = loaded();
    let red_sha = inject(&mut tx, source, CardKind::Sha, Suit::Heart);
    assert_eq!(
        dispatch(&mut tx, source, red_sha, vec![target]).unwrap_err(),
        EffectError::ChoiceRequested
    );
    let state = accept(tx, owner, 2);

    let mut tx = Transaction::begin(state);
    assert_eq!(
        dispatch(&mut tx, source, red_sha, vec![target]).unwrap_err(),
        EffectError::ChoiceRequested
    );
    let state = accept(tx, owner, 3);

    let mut tx = Transaction::begin(state);
    let other_red = inject(&mut tx, source, CardKind::Other, Suit::Diamond);
    assert!(dispatch(&mut tx, source, other_red, vec![target])
        .unwrap()
        .is_empty());
    assert!(dispatch(&mut tx, source, red_sha, vec![target])
        .unwrap()
        .is_empty());

    run_effects(
        &mut tx,
        vec![EffectRequest::SetTurnState {
            current: source,
            phase: Phase::Prepare.to_u8(),
        }],
    )
    .unwrap();
    assert_eq!(
        dispatch(&mut tx, source, red_sha, vec![target]).unwrap_err(),
        EffectError::ChoiceRequested
    );
}
