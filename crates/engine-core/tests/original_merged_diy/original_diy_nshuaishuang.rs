//! Golden for DIY「怀霜」有桃才获得并失去体力的语义。

use engine_api::{
    CardDefId, CardId, Command, CommandKind, CommandResult, DomainEvent, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit, TurnPhaseTag,
};
use engine_core::effect::run_effects;
use engine_core::state::{Card, CardKind, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use engine_core::Engine;
use rule_api::EffectRequest;

const NSHUAISHUANG: u32 = 3_540_859_495;
const PACKAGE: &[u8] = include_bytes!("../../../../content/original/ported/diy/diy.npack.json");

#[test]
fn huaishuang_loses_hp_only_after_gaining_a_tao() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == NSHUAISHUANG && skill.locked));
    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(NSHUAISHUANG as u64),
            kind: CommandKind::StartGame {
                player_count: 2,
                rng_seed: NSHUAISHUANG as u64,
                ruleset: "identity-standard".into(),
            },
        }),
        CommandResult::Accepted { .. }
    ));
    let owner = engine.state().turn.current;
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
        skill_def: SkillDefId::new(NSHUAISHUANG),
    });
    run_effects(&mut tx, effects).unwrap();

    let draw = tx
        .state()
        .zones
        .find(ZoneKind::DrawPile, PlayerId::NONE)
        .unwrap();
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    let removed = tx
        .state()
        .zones
        .find(ZoneKind::Removed, PlayerId::NONE)
        .unwrap();
    for zone in [draw, discard] {
        let cards: Vec<_> = tx
            .state()
            .zones
            .get(zone)
            .unwrap()
            .cards
            .iter()
            .copied()
            .filter(|card| tx.state().cards.get(*card).unwrap().kind == CardKind::Tao)
            .collect();
        if !cards.is_empty() {
            run_effects(
                &mut tx,
                vec![EffectRequest::MoveCards {
                    cards,
                    to: removed,
                    from: Some(zone),
                }],
            )
            .unwrap();
        }
    }

    let hp = tx.state().players.get(owner).unwrap().hp;
    let hand = tx.state().hand_count(owner);
    fire_end_phase(&mut tx, owner);
    assert_eq!(tx.state().players.get(owner).unwrap().hp, hp);
    assert_eq!(tx.state().hand_count(owner), hand);

    let tao = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(NSHUAISHUANG + 1),
        kind: CardKind::Tao,
        suit: Suit::Heart,
        rank: 12,
        weapon_range: 0,
        zone: discard,
        owner: PlayerId::NONE,
        face_up: true,
    });
    tx.state_mut()
        .zones
        .get_mut(discard)
        .unwrap()
        .cards
        .push(tao);
    fire_end_phase(&mut tx, owner);
    assert!(tx.state().hand_cards(owner).contains(&tao));
    assert_eq!(tx.state().players.get(owner).unwrap().hp, hp - 1);
}

fn fire_end_phase(tx: &mut Transaction, owner: PlayerId) {
    let effects = dispatch_for_events(
        tx,
        &[DomainEvent::TurnStateChanged {
            current: owner,
            phase: TurnPhaseTag::End,
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    run_effects(tx, effects).unwrap();
}
