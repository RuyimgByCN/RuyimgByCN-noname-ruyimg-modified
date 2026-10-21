//! Golden for OFFLINE「武库」的装备牌与三枚标记上限语义。

use engine_api::{
    CardDefId, CardId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey,
    PlayerId, SkillDefId, Suit,
};
use engine_core::effect::run_effects;
use engine_core::state::{Card, CardKind, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::EffectRequest;

const PKWUKU: u32 = 3_003_644_541;
const PACKAGE: &[u8] =
    include_bytes!("../../../../content/original/ported/offline/offline.npack.json");

fn inject(tx: &mut Transaction, owner: PlayerId, kind: CardKind, rank: u8) -> CardId {
    let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(98_000 + u32::from(rank)),
        kind,
        suit: Suit::Club,
        rank,
        weapon_range: kind.is_equip().then_some(2).unwrap_or(0),
        zone: hand,
        owner,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(hand).unwrap().cards.push(card);
    card
}

fn mark_count(tx: &Transaction, owner: PlayerId) -> i32 {
    tx.state()
        .players
        .get(owner)
        .unwrap()
        .marks
        .iter()
        .find(|(mark, _)| *mark == PKWUKU)
        .map_or(0, |(_, count)| *count)
}

#[test]
fn only_equipment_uses_add_wuku_and_the_mark_caps_at_three() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(0),
            kind: CommandKind::StartGame {
                player_count: 2,
                rng_seed: 20_260_715,
                ruleset: "identity-standard".into(),
            },
        }),
        CommandResult::Accepted { .. }
    ));
    let owner = engine.state().turn.current;
    let user = engine
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
        skill_def: SkillDefId::new(PKWUKU),
    });
    run_effects(&mut tx, effects).unwrap();

    let sha = inject(&mut tx, user, CardKind::Sha, 1);
    run_effects(
        &mut tx,
        vec![EffectRequest::UseCard {
            player: user,
            card: sha,
            targets: vec![owner],
        }],
    )
    .unwrap();
    assert_eq!(mark_count(&tx, owner), 0);

    for rank in 2..=5 {
        let equip = inject(&mut tx, user, CardKind::Weapon, rank);
        run_effects(
            &mut tx,
            vec![EffectRequest::UseCard {
                player: user,
                card: equip,
                targets: vec![user],
            }],
        )
        .unwrap();
    }
    assert_eq!(mark_count(&tx, owner), 3);
}
