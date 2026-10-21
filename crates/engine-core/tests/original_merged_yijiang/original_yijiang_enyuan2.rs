//! Golden for 一将成名法正「恩怨」的回复来源语义。

use engine_api::{Command, CommandKind, ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId};
use engine_core::effect::run_effects;
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::EffectRequest;

const ENYUAN2: u32 = 3_002_282_592;
const PACKAGE: &[u8] =
    include_bytes!("../../../../content/original/ported/yijiang/yijiang.npack.json");

fn isolated_state() -> (Transaction, PlayerId, PlayerId) {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == ENYUAN2 && skill.locked));

    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    engine.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(3_002_282_592),
        kind: CommandKind::StartGame {
            player_count: 3,
            rng_seed: 20_260_714,
            ruleset: "identity-standard".into(),
        },
    });
    let owner = engine.state().turn.current;
    let source = engine
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
        skill_def: SkillDefId::new(ENYUAN2),
    });
    run_effects(&mut tx, effects).unwrap();
    (tx, owner, source)
}

#[test]
fn other_living_recovery_source_draws_actual_recovered_amount_only() {
    let (mut tx, owner, source) = isolated_state();
    tx.state_mut().players.get_mut(owner).unwrap().hp = 1;
    let source_hand = tx.state().hand_count(source);

    run_effects(
        &mut tx,
        vec![EffectRequest::RecoverHp {
            source,
            player: owner,
            amount: 2,
        }],
    )
    .unwrap();
    assert_eq!(tx.state().hand_count(source), source_hand + 2);

    tx.state_mut().players.get_mut(owner).unwrap().hp = 1;
    let owner_hand = tx.state().hand_count(owner);
    run_effects(
        &mut tx,
        vec![EffectRequest::RecoverHp {
            source: owner,
            player: owner,
            amount: 1,
        }],
    )
    .unwrap();
    assert_eq!(tx.state().hand_count(owner), owner_hand);

    tx.state_mut().players.get_mut(owner).unwrap().hp = 1;
    tx.state_mut().players.get_mut(source).unwrap().alive = false;
    let source_hand = tx.state().hand_count(source);
    run_effects(
        &mut tx,
        vec![EffectRequest::RecoverHp {
            source,
            player: owner,
            amount: 1,
        }],
    )
    .unwrap();
    assert_eq!(tx.state().hand_count(source), source_hand);
}
