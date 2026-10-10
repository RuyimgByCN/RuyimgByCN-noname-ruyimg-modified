//! Golden for SP「蔽日」的首个非摸牌得牌语义。

use engine_api::{
    Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId,
};
use engine_core::effect::run_effects;
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::EffectRequest;

const OLBIRI: u32 = 3_834_015_931;
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/sp/sp.npack.json");

#[test]
fn draws_only_for_the_first_non_draw_gain_each_turn() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(0),
            kind: CommandKind::StartGame {
                player_count: 3,
                rng_seed: 20_260_715,
                ruleset: "identity-standard".into(),
            },
        }),
        CommandResult::Accepted { .. }
    ));
    let owner = engine.state().turn.current;
    let others: Vec<_> = engine
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .filter(|player| *player != owner)
        .collect();
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
        skill_def: SkillDefId::new(OLBIRI),
    });
    run_effects(&mut tx, effects).unwrap();

    let before = tx.state().hand_count(owner);
    run_effects(
        &mut tx,
        vec![EffectRequest::DrawCards {
            player: others[0],
            count: 1,
        }],
    )
    .unwrap();
    assert_eq!(tx.state().hand_count(owner), before);

    run_effects(
        &mut tx,
        vec![EffectRequest::TakeFromHand {
            from: others[0],
            to: others[1],
        }],
    )
    .unwrap();
    assert_eq!(tx.state().hand_count(owner), before + 1);

    run_effects(
        &mut tx,
        vec![EffectRequest::TakeFromHand {
            from: others[1],
            to: others[0],
        }],
    )
    .unwrap();
    assert_eq!(tx.state().hand_count(owner), before + 1);
}
