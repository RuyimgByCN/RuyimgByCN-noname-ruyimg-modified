//! Golden coverage for 标吕范「典封」and the typed last-equipment-loss filter.

use engine_api::{
    ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey, PlayerId,
    SkillDefId,
};
use engine_core::command::apply_command;
use engine_core::effect::{move_one_card, run_effects, EffectError};
use engine_core::state::{CardKind, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::EffectRequest;

const STDDIANFENG: u32 = 3_414_143_685;
const PACKAGE: &[u8] =
    include_bytes!("../../../../content/original/ported/sixiang/sixiang.npack.json");

fn setup(equipment_count: usize) -> (Transaction, PlayerId, PlayerId) {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(STDDIANFENG as u64),
            kind: CommandKind::StartGame {
                player_count: 2,
                rng_seed: STDDIANFENG as u64,
                ruleset: "identity-standard".into(),
            },
        }),
        CommandResult::Accepted { .. }
    ));
    let players = engine.state().players.alive_by_seat();
    let (owner, target) = (players[0], players[1]);
    let mut tx = Transaction::begin(engine.state().clone());
    let mut setup: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    setup.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(STDDIANFENG),
    });
    run_effects(&mut tx, setup).unwrap();

    let hand = tx.state().zones.find(ZoneKind::Hand, target).unwrap();
    let equip = tx.state().zones.find(ZoneKind::Equip, target).unwrap();
    for card in tx
        .state()
        .hand_cards(target)
        .into_iter()
        .take(equipment_count)
        .collect::<Vec<_>>()
    {
        tx.state_mut().cards.get_mut(card).unwrap().kind = CardKind::Weapon;
        move_one_card(&mut tx, card, hand, equip, target, true).unwrap();
    }
    (tx, owner, target)
}

fn move_equipment_to_discard(
    tx: &mut Transaction,
    target: PlayerId,
    count: usize,
) -> Result<Vec<engine_api::DomainEvent>, EffectError> {
    let equip = tx.state().zones.find(ZoneKind::Equip, target).unwrap();
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    let cards = tx
        .state()
        .zones
        .get(equip)
        .unwrap()
        .cards
        .iter()
        .copied()
        .take(count)
        .collect();
    run_effects(
        tx,
        vec![EffectRequest::MoveCards {
            cards,
            to: discard,
            from: Some(equip),
        }],
    )
}

#[test]
fn committed_package_declares_precise_optional_trigger() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 562);
    let trigger = package
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == STDDIANFENG)
        .unwrap();
    assert_eq!(trigger.event, "cards_moved");
    assert_eq!(trigger.filter, "owner_alive_event_player_from_equip_empty");
    assert!(trigger.optional);
}

#[test]
fn losing_the_last_equipment_offers_owner_draw() {
    let (mut tx, owner, target) = setup(1);
    let hand_before = tx.state().hand_count(owner);
    assert_eq!(
        move_equipment_to_discard(&mut tx, target, 1).unwrap_err(),
        EffectError::ChoiceRequested
    );
    assert_eq!(tx.state().resolution.pending_observer, owner.raw());

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
    assert_eq!(state.hand_count(owner), hand_before + 1);
}

#[test]
fn remaining_equipment_and_unrelated_moves_do_not_trigger() {
    let (mut tx, owner, target) = setup(2);
    let hand_before = tx.state().hand_count(owner);
    move_equipment_to_discard(&mut tx, target, 1).unwrap();
    assert!(!tx.state().has_pending_choice());
    assert_eq!(tx.state().hand_count(owner), hand_before);

    let (mut tx, owner, target) = setup(0);
    let hand_before = tx.state().hand_count(owner);
    let hand = tx.state().zones.find(ZoneKind::Hand, target).unwrap();
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    let card = tx.state().hand_cards(target)[0];
    run_effects(
        &mut tx,
        vec![EffectRequest::MoveCards {
            cards: vec![card],
            to: discard,
            from: Some(hand),
        }],
    )
    .unwrap();
    assert!(!tx.state().has_pending_choice());
    assert_eq!(tx.state().hand_count(owner), hand_before);
}
