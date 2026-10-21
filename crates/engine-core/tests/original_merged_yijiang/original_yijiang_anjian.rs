use engine_api::{
    CardDefId, CardId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey,
    PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::run_effects;
use engine_core::state::{Card, CardKind, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::EffectRequest;

const PACKAGE: &[u8] =
    include_bytes!("../../../../content/original/ported/yijiang/yijiang.npack.json");
const ANJIAN: u32 = 3_402_374_234;

#[test]
fn anjian_uses_the_targets_attack_range_for_logical_sha_damage() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(ANJIAN as u64),
            kind: CommandKind::StartGame {
                player_count: 4,
                rng_seed: ANJIAN as u64,
                ruleset: "identity-standard".into(),
            },
        }),
        CommandResult::Accepted { .. }
    ));
    let source = engine.state().turn.current;
    let seats = engine.state().players.alive_by_seat();
    let source_index = seats.iter().position(|player| *player == source).unwrap();
    let adjacent = seats[(source_index + 1) % seats.len()];
    let target = seats[(source_index + 2) % seats.len()];
    let mut tx = Transaction::begin(engine.state().clone());
    tx.state_mut().revision = 1;
    tx.state_mut().turn.phase = Phase::Play;
    let mut isolate: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    isolate.push(EffectRequest::AddSkill {
        player: source,
        skill_def: SkillDefId::new(ANJIAN),
    });
    run_effects(&mut tx, isolate).unwrap();
    assert_eq!(
        engine_core::query::sha_damage_outside_target_range_for(tx.state(), source, adjacent, 1),
        1
    );
    assert_eq!(
        engine_core::query::sha_damage_outside_target_range_for(tx.state(), source, target, 1),
        2
    );

    let source_equip = tx.state().zones.find(ZoneKind::Equip, source).unwrap();
    let weapon = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(ANJIAN + 1),
        kind: CardKind::Weapon,
        suit: Suit::Club,
        rank: 5,
        weapon_range: 2,
        zone: source_equip,
        owner: source,
        face_up: true,
    });
    tx.state_mut()
        .zones
        .get_mut(source_equip)
        .unwrap()
        .cards
        .push(weapon);
    run_effects(
        &mut tx,
        vec![EffectRequest::DiscardFromHand {
            player: target,
            count: u32::MAX,
        }],
    )
    .unwrap();
    tx.state_mut().players.get_mut(target).unwrap().hp = 10;
    tx.state_mut().players.get_mut(target).unwrap().max_hp = 10;
    let source_hand = tx.state().zones.find(ZoneKind::Hand, source).unwrap();
    let sha = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(ANJIAN + 2),
        kind: CardKind::Sha,
        suit: Suit::Spade,
        rank: 7,
        weapon_range: 0,
        zone: source_hand,
        owner: source,
        face_up: false,
    });
    tx.state_mut()
        .zones
        .get_mut(source_hand)
        .unwrap()
        .cards
        .push(sha);

    let (state, result) = apply_command(
        tx.state().clone(),
        Command {
            actor: source,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::PlayCard {
                card: sha,
                targets: vec![target],
                card_targets: vec![],
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(target).unwrap().hp, 8);
}
