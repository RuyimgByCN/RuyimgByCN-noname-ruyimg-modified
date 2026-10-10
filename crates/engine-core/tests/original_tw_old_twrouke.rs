//! Golden for TW「柔克」的多牌获得与摸牌阶段排除语义。

use engine_api::{
    Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId,
};
use engine_core::effect::run_effects;
use engine_core::state::Phase;
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::EffectRequest;

const OLD_TWROUKE: u32 = 3_805_481_666;
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/tw/tw.npack.json");

fn isolated_state() -> (Transaction, PlayerId, PlayerId) {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(
        package
            .trigger_index
            .iter()
            .filter(|trigger| trigger.skill_def_id == OLD_TWROUKE)
            .count(),
        2
    );

    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(OLD_TWROUKE as u64),
            kind: CommandKind::StartGame {
                player_count: 2,
                rng_seed: OLD_TWROUKE as u64,
                ruleset: "identity-standard".into(),
            },
        }),
        CommandResult::Accepted { .. }
    ));

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
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(OLD_TWROUKE),
    });
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.phase = Phase::Play;
    (tx, owner, other)
}

#[test]
fn draws_once_for_each_multi_card_gain_outside_the_owners_draw_phase() {
    let (mut tx, owner, other) = isolated_state();

    let hand = tx.state().hand_count(owner);
    run_effects(
        &mut tx,
        vec![EffectRequest::DrawCards {
            player: owner,
            count: 2,
        }],
    )
    .unwrap();
    assert_eq!(tx.state().hand_count(owner), hand + 3);

    tx.state_mut().turn.phase = Phase::Draw;
    let hand = tx.state().hand_count(owner);
    run_effects(
        &mut tx,
        vec![EffectRequest::DrawCards {
            player: owner,
            count: 2,
        }],
    )
    .unwrap();
    assert_eq!(tx.state().hand_count(owner), hand + 2);

    tx.state_mut().turn.phase = Phase::Play;
    let hand = tx.state().hand_count(owner);
    run_effects(
        &mut tx,
        vec![EffectRequest::DrawCards {
            player: other,
            count: 2,
        }],
    )
    .unwrap();
    assert_eq!(tx.state().hand_count(owner), hand);

    run_effects(
        &mut tx,
        vec![EffectRequest::DrawCards {
            player: owner,
            count: 1,
        }],
    )
    .unwrap();
    assert_eq!(tx.state().hand_count(owner), hand + 1);

    let from = tx
        .state()
        .zones
        .find(engine_core::state::ZoneKind::Hand, other)
        .unwrap();
    let to = tx
        .state()
        .zones
        .find(engine_core::state::ZoneKind::Hand, owner)
        .unwrap();
    let cards = tx.state().hand_cards(other)[..2].to_vec();
    let hand = tx.state().hand_count(owner);
    run_effects(
        &mut tx,
        vec![EffectRequest::MoveCards {
            cards,
            to,
            from: Some(from),
        }],
    )
    .unwrap();
    assert_eq!(tx.state().hand_count(owner), hand + 3);
}
