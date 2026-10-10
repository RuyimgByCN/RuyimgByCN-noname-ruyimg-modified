//! Golden tests for KEY「奉承」rewritten from the legacy JavaScript semantics.

use engine_api::{
    CardMoveReason, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey,
    PlayerId, SkillDefId,
};
use engine_core::effect::run_effects;
use engine_core::state::ZoneKind;
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::EffectRequest;

const NODA_FENGCHENG: u32 = 2_452_779_390;
const SOURCE: &str = include_str!("../../../content/original/ported/key/noda_fengcheng.skill");
const KEY: &[u8] = include_bytes!("../../../content/original/ported/key/key.npack.json");

fn start(seed: u64) -> Command {
    Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(0),
        kind: CommandKind::StartGame {
            player_count: 2,
            rng_seed: seed,
            ruleset: "identity-standard".into(),
        },
    }
}

fn setup() -> (Transaction, PlayerId, PlayerId) {
    let package = content_compiler::compile_source_named("noda_fengcheng", SOURCE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    assert!(matches!(
        engine.submit(start(2)),
        CommandResult::Accepted { .. }
    ));
    let players: Vec<_> = engine
        .state()
        .players
        .iter_live()
        .map(|player| player.id)
        .collect();
    let giver = players[0];
    let owner = players[1];
    let mut tx = Transaction::begin(engine.state().clone());
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(NODA_FENGCHENG),
        }],
    )
    .unwrap();
    (tx, giver, owner)
}

#[test]
fn committed_package_contains_the_locked_rewrite() {
    let committed = content_schema::ContentPackage::from_json_bytes(KEY).unwrap();
    let compiled = content_compiler::compile_source_named("noda_fengcheng", SOURCE).unwrap();
    let expected = &compiled.modules[0];
    let actual = committed
        .modules
        .iter()
        .find(|module| module.name == expected.name)
        .unwrap();
    assert_eq!(actual.logic_hash, expected.logic_hash);
    assert_eq!(compiled.manifest.rule_vm_abi_version, 3);
    assert_eq!(compiled.trigger_index[0].event, "cards_moved");
    assert_eq!(
        compiled.trigger_index[0].filter,
        "owner_received_gift_from_other"
    );
    assert!(!compiled.trigger_index[0].optional);
}

#[test]
fn voluntary_gift_draws_one_for_the_giver() {
    let (mut tx, giver, owner) = setup();
    let card = tx.state().hand_cards(giver)[0];
    let from = tx.state().cards.get(card).unwrap().zone;
    let to = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let giver_before = tx.state().hand_count(giver);
    let owner_before = tx.state().hand_count(owner);

    let events = run_effects(
        &mut tx,
        vec![EffectRequest::GiveCards {
            cards: vec![card],
            to,
            from,
        }],
    )
    .unwrap();

    assert_eq!(tx.state().hand_count(giver), giver_before);
    assert_eq!(tx.state().hand_count(owner), owner_before + 1);
    assert!(events.iter().any(|event| matches!(
        event,
        engine_api::DomainEvent::CardsMoved {
            cards,
            reason: CardMoveReason::Give,
            ..
        } if cards == &vec![card]
    )));
    assert!(events.iter().any(|event| matches!(
        event,
        engine_api::DomainEvent::CardsDrawn { player, cards }
            if *player == giver && cards.len() == 1
    )));
}

#[test]
fn taking_the_same_card_does_not_count_as_a_gift() {
    let (mut tx, giver, owner) = setup();
    let card = tx.state().hand_cards(giver)[0];
    let from = tx.state().cards.get(card).unwrap().zone;
    let to = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let giver_before = tx.state().hand_count(giver);
    let owner_before = tx.state().hand_count(owner);

    let events = run_effects(
        &mut tx,
        vec![EffectRequest::MoveCards {
            cards: vec![card],
            to,
            from: Some(from),
        }],
    )
    .unwrap();

    assert_eq!(tx.state().hand_count(giver), giver_before - 1);
    assert_eq!(tx.state().hand_count(owner), owner_before + 1);
    assert!(!events.iter().any(|event| matches!(
        event,
        engine_api::DomainEvent::CardsDrawn { player, .. } if *player == giver
    )));
}
