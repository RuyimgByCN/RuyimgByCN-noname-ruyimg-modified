//! Golden for target-side nature-specific damage cancellation.

use engine_api::{Command, CommandKind, ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId};
use engine_core::effect::run_effects;
use engine_core::state::CardKind;
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::EffectRequest;

const CLBSHUIYONG: u32 = 4_055_999_596;
const SHIXIN: u32 = 3_071_284_185;
const PSYUHUO: u32 = 2_658_654_582;
const YIRAN: u32 = 3_082_356_765;
const OLYOUJI: u32 = 2_389_723_895;
const COLLAB: &[u8] = include_bytes!("../../../content/original/ported/collab/collab.npack.json");
const MOBILE: &[u8] = include_bytes!("../../../content/original/ported/mobile/mobile.npack.json");
const OFFLINE: &[u8] =
    include_bytes!("../../../content/original/ported/offline/offline.npack.json");

fn isolated(package: &[u8], skill: u32, seed: u64) -> (Transaction, PlayerId, PlayerId) {
    let package = content_schema::ContentPackage::from_json_bytes(package).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    engine.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(seed),
        kind: CommandKind::StartGame {
            player_count: 2,
            rng_seed: seed,
            ruleset: "identity-standard".into(),
        },
    });
    let owner = engine.state().turn.current;
    let other = engine
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
        .map(|active| EffectRequest::RemoveSkill { skill: active.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(skill),
    });
    run_effects(&mut tx, effects).unwrap();
    for player in [owner, other] {
        let state = tx.state_mut().players.get_mut(player).unwrap();
        state.hp = 10;
        state.max_hp = 10;
    }
    (tx, owner, other)
}

fn assert_fire_only(mut tx: Transaction, owner: PlayerId, other: PlayerId) {
    run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source: other,
            target: owner,
            amount: 1,
            is_lose_hp: false,
            nature: 1,
        }],
    )
    .unwrap();
    assert_eq!(tx.state().players.get(owner).unwrap().hp, 10);

    run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source: other,
            target: owner,
            amount: 1,
            is_lose_hp: false,
            nature: 0,
        }],
    )
    .unwrap();
    run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source: other,
            target: owner,
            amount: 1,
            is_lose_hp: false,
            nature: 2,
        }],
    )
    .unwrap();
    assert_eq!(tx.state().players.get(owner).unwrap().hp, 8);

    run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source: owner,
            target: other,
            amount: 1,
            is_lose_hp: false,
            nature: 1,
        }],
    )
    .unwrap();
    assert_eq!(tx.state().players.get(other).unwrap().hp, 9);
}

#[test]
fn clbshuiyong_cancels_only_fire_damage_received_by_the_owner() {
    let (tx, owner, other) = isolated(COLLAB, CLBSHUIYONG, 20_260_720);
    assert_fire_only(tx, owner, other);
}

#[test]
fn shixin_uses_the_same_target_side_fire_immunity() {
    let (tx, owner, other) = isolated(MOBILE, SHIXIN, 20_260_721);
    assert_fire_only(tx, owner, other);
}

#[test]
fn psyuhuo_also_excludes_fire_sha_and_huogong_from_discard_to_limit() {
    let (tx, owner, other) = isolated(OFFLINE, PSYUHUO, 20_260_722);
    assert_fire_only(tx, owner, other);

    let (mut tx, owner, _) = isolated(OFFLINE, PSYUHUO, 20_260_723);
    let cards = tx.state().hand_cards(owner);
    assert!(cards.len() >= 3);
    tx.state_mut().cards.get_mut(cards[0]).unwrap().kind = CardKind::ShaFire;
    tx.state_mut().cards.get_mut(cards[1]).unwrap().kind = CardKind::TrickHuogong;
    tx.state_mut().cards.get_mut(cards[2]).unwrap().kind = CardKind::Shan;

    assert!(engine_core::query::card_ignored_for_hand_limit(
        tx.state(),
        owner,
        cards[0]
    ));
    assert!(engine_core::query::card_ignored_for_hand_limit(
        tx.state(),
        owner,
        cards[1]
    ));
    assert!(!engine_core::query::card_ignored_for_hand_limit(
        tx.state(),
        owner,
        cards[2]
    ));
    let discardable = engine_core::query::discard_to_limit_tokens(tx.state(), owner);
    assert!(!discardable.contains(&cards[0]));
    assert!(!discardable.contains(&cards[1]));
    assert!(discardable.contains(&cards[2]));
}

#[test]
fn yiran_adds_one_only_to_fire_damage_received_by_the_owner() {
    let (mut tx, owner, other) = isolated(COLLAB, YIRAN, 20_260_724);
    for (nature, lose_hp, expected_hp) in [
        (rule_api::damage_nature::NORMAL, false, 9),
        (rule_api::damage_nature::THUNDER, false, 8),
        (rule_api::damage_nature::FIRE, false, 6),
        (rule_api::damage_nature::FIRE, true, 5),
    ] {
        run_effects(
            &mut tx,
            vec![EffectRequest::DealDamage {
                source: other,
                target: owner,
                amount: 1,
                is_lose_hp: lose_hp,
                nature,
            }],
        )
        .unwrap();
        assert_eq!(tx.state().players.get(owner).unwrap().hp, expected_hp);
    }

    run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source: owner,
            target: other,
            amount: 1,
            is_lose_hp: false,
            nature: rule_api::damage_nature::FIRE,
        }],
    )
    .unwrap();
    assert_eq!(tx.state().players.get(other).unwrap().hp, 9);
}

#[test]
fn olyouji_adds_the_round_number_to_draws_with_a_cap_of_five() {
    let (mut tx, owner, _) = isolated(COLLAB, OLYOUJI, 20_260_725);
    for (round, expected) in [(0, 2), (1, 3), (4, 6), (5, 7), (9, 7)] {
        tx.state_mut().turn.round = round;
        assert_eq!(
            engine_core::query::draw_count_for(tx.state(), owner),
            expected
        );
    }
}
