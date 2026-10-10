//! Golden for SP2「翦羽」：拥有者回合内按失去装备的其他角色摸牌。

use engine_api::{Command, CommandKind, ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId};
use engine_core::effect::run_effects;
use engine_core::state::{CardKind, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::EffectRequest;

const STARJIANYU: u32 = 2_340_348_064;
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/sp2/sp2.npack.json");

#[test]
fn draws_once_when_another_player_loses_equipment_during_owners_turn() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    engine.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(20_260_715),
        kind: CommandKind::StartGame {
            player_count: 3,
            rng_seed: 20_260_715,
            ruleset: "identity-standard".into(),
        },
    });
    let players = engine.state().players.alive_by_seat();
    let [owner, other, third] = players.as_slice() else {
        panic!("expected three players");
    };
    let (owner, other, third) = (*owner, *other, *third);
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
        skill_def: SkillDefId::new(STARJIANYU),
    });
    run_effects(&mut tx, setup).unwrap();

    let other_hand = tx.state().zones.find(ZoneKind::Hand, other).unwrap();
    let other_equip = tx.state().zones.find(ZoneKind::Equip, other).unwrap();
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    let equipment = tx.state().hand_cards(other)[..2].to_vec();
    tx.state_mut().cards.get_mut(equipment[0]).unwrap().kind = CardKind::Weapon;
    tx.state_mut().cards.get_mut(equipment[1]).unwrap().kind = CardKind::Armor;
    run_effects(
        &mut tx,
        vec![EffectRequest::MoveCards {
            cards: equipment.clone(),
            to: other_equip,
            from: Some(other_hand),
        }],
    )
    .unwrap();
    let hand = tx.state().hand_count(owner);

    run_effects(
        &mut tx,
        vec![EffectRequest::MoveCards {
            cards: equipment,
            to: discard,
            from: Some(other_equip),
        }],
    )
    .unwrap();
    assert_eq!(tx.state().hand_count(owner), hand + 1);

    let third_hand = tx.state().zones.find(ZoneKind::Hand, third).unwrap();
    let third_equip = tx.state().zones.find(ZoneKind::Equip, third).unwrap();
    let third_card = tx.state().hand_cards(third)[0];
    tx.state_mut().cards.get_mut(third_card).unwrap().kind = CardKind::Weapon;
    run_effects(
        &mut tx,
        vec![
            EffectRequest::MoveCards {
                cards: vec![third_card],
                to: third_equip,
                from: Some(third_hand),
            },
            EffectRequest::SetTurnState {
                current: other,
                phase: Phase::Play.to_u8(),
            },
            EffectRequest::MoveCards {
                cards: vec![third_card],
                to: discard,
                from: Some(third_equip),
            },
        ],
    )
    .unwrap();
    assert_eq!(tx.state().hand_count(owner), hand + 1);
}
