//! Golden for MOBILE「进讨」前两张杀的顺序修正。

use engine_api::{
    CardDefId, CardId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey,
    PlayerId, SkillDefId, Suit,
};
use engine_core::effect::run_effects;
use engine_core::state::{turn_flag, Card, CardKind, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::EffectRequest;

const XINJINTAO: u32 = 2_174_751_337;
const XIN_WUBAN: u32 = 3_601_555_604;
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/mobile/mobile.npack.json");

#[test]
fn first_sha_is_unrespondable_and_second_sha_gains_damage() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(
        package
            .character_index
            .iter()
            .find(|character| character.id == XIN_WUBAN)
            .unwrap()
            .skills,
        [XINJINTAO]
    );

    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(XINJINTAO as u64),
            kind: CommandKind::StartGame {
                player_count: 2,
                rng_seed: XINJINTAO as u64,
                ruleset: "identity-standard".into(),
            },
        }),
        CommandResult::Accepted { .. }
    ));

    let owner = engine.state().turn.current;
    let target = engine
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .find(|player| *player != owner)
        .unwrap();
    let mut tx = Transaction::begin(engine.state().clone());
    tx.state_mut().turn.phase = Phase::Play;
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(XINJINTAO),
    });
    run_effects(&mut tx, effects).unwrap();

    let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let sha = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(XINJINTAO),
        kind: CardKind::Sha,
        suit: Suit::Spade,
        rank: 7,
        weapon_range: 0,
        zone: hand,
        owner,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(hand).unwrap().cards.push(sha);

    assert_eq!(engine_core::query::sha_use_limit_for(tx.state(), owner), 2);
    assert!(engine_core::query::card_distance_unlimited_for_target(
        tx.state(),
        owner,
        sha,
        target
    ));

    tx.state_mut().turn.set_flag(turn_flag::SHA_PLAYED, 1);
    assert!(engine_core::query::sha_unrespondable_for_card_target(
        tx.state(),
        owner,
        target,
        sha
    ));
    assert_eq!(
        engine_core::query::sha_damage_for_card(tx.state(), owner, sha, 1),
        1
    );

    tx.state_mut().turn.set_flag(turn_flag::SHA_PLAYED, 2);
    assert!(!engine_core::query::sha_unrespondable_for_card_target(
        tx.state(),
        owner,
        target,
        sha
    ));
    assert_eq!(
        engine_core::query::sha_damage_for_card(tx.state(), owner, sha, 1),
        2
    );

    tx.state_mut().turn.set_flag(turn_flag::SHA_PLAYED, 3);
    assert!(!engine_core::query::sha_unrespondable_for_card_target(
        tx.state(),
        owner,
        target,
        sha
    ));
    assert_eq!(
        engine_core::query::sha_damage_for_card(tx.state(), owner, sha, 1),
        1
    );
}
