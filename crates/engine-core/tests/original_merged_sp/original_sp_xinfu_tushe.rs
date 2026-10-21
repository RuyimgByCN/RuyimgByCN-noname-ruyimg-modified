//! Golden for SP `xinfu_tushe`, rewritten from legacy JavaScript.

use engine_api::{
    CardId, ChoiceId, Command, CommandKind, CommandResult, DomainEvent, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{CardKind, GameState};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const XINFU_TUSHE: u32 = 2_236_248_008;
const PACKAGE: &[u8] = include_bytes!("../../../../content/original/ported/sp/sp.npack.json");

fn started(seed: u64) -> (Transaction, PlayerId, Vec<PlayerId>) {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 4, seed, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content)
        .load_package(&package)
        .unwrap();
    let owner = tx.state().turn.current;
    let targets = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .filter(|player| *player != owner)
        .collect();
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(XINFU_TUSHE),
        }],
    )
    .unwrap();
    (tx, owner, targets)
}

fn set_hand_kind(tx: &mut Transaction, owner: PlayerId, kind: CardKind) -> Vec<CardId> {
    let hand = tx.state().hand_cards(owner);
    for card in &hand {
        tx.state_mut().cards.get_mut(*card).unwrap().kind = kind;
    }
    hand
}

#[test]
fn xinfu_tushe_optionally_draws_full_target_count_once() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let trigger = package
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == XINFU_TUSHE)
        .unwrap();
    assert_eq!(trigger.event, "card_used");
    assert_eq!(
        trigger.filter,
        "owner_source_non_equip_no_basic_hand_has_targets"
    );
    assert!(trigger.optional);

    let (mut tx, owner, targets) = started(2_026_071_523);
    let card = set_hand_kind(&mut tx, owner, CardKind::TrickWuzhong)[0];
    let hand = tx.state().hand_count(owner);
    assert_eq!(
        dispatch_for_events(
            &mut tx,
            &[DomainEvent::CardUsed {
                player: owner,
                card,
                targets: targets.clone(),
            }],
            &mut TriggerFireRecord::new(),
        )
        .unwrap_err(),
        EffectError::ChoiceRequested
    );
    let (state, result) = apply_command(
        tx.state().clone(),
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(tx.state().revision),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::ResolveChoice {
                choice_id: ChoiceId::new(tx.state().resolution.pending_choice_id),
                selections: vec![1],
                cancelled: false,
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(owner), hand + targets.len() as u32);
}

#[test]
fn xinfu_tushe_rejects_basic_hand_equipment_and_targetless_uses() {
    let (mut tx, owner, targets) = started(2_026_071_524);
    let hand = set_hand_kind(&mut tx, owner, CardKind::TrickWuzhong);
    tx.state_mut().cards.get_mut(hand[1]).unwrap().kind = CardKind::Tao;
    assert!(dispatch_for_events(
        &mut tx,
        &[DomainEvent::CardUsed {
            player: owner,
            card: hand[0],
            targets: targets.clone(),
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap()
    .is_empty());

    let (mut tx, owner, targets) = started(2_026_071_525);
    let card = set_hand_kind(&mut tx, owner, CardKind::Weapon)[0];
    assert!(dispatch_for_events(
        &mut tx,
        &[DomainEvent::CardUsed {
            player: owner,
            card,
            targets,
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap()
    .is_empty());

    let (mut tx, owner, _) = started(2_026_071_526);
    let card = set_hand_kind(&mut tx, owner, CardKind::TrickWuzhong)[0];
    assert!(dispatch_for_events(
        &mut tx,
        &[DomainEvent::CardUsed {
            player: owner,
            card,
            targets: vec![],
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap()
    .is_empty());
}
