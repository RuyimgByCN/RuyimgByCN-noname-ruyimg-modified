//! Golden for sp 法箓 (xinfu_falu)：locked —— 开局获得四花色「法箓」标记各 1 枚
//! （批 183 花色化终版；supersede 批 179 的统一标记口径）。

use engine_api::{
    Command, CommandKind, CommandResult, DomainEvent, ExpectedRevision, IdempotencyKey, PlayerId,
    SkillDefId,
};
use engine_core::effect::run_effects;
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use engine_core::Engine;
use rule_api::EffectRequest;

const XINFU_FALU: u32 = 3_540_560_358;
const FALU_SUIT_MARKS: [u32; 4] = [
    3_540_560_359, // spade
    3_540_560_360, // heart
    3_540_560_361, // club
    3_540_560_362, // diamond
];
const PACKAGE: &[u8] = include_bytes!("../../../../content/original/ported/sp/sp.npack.json");

fn start() -> (Engine, PlayerId) {
    let mut engine = Engine::new();
    let pkg = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    engine.load_package(&pkg).unwrap();
    let result = engine.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(0),
        kind: CommandKind::StartGame {
            player_count: 2,
            rng_seed: 20_262_001,
            ruleset: "identity-standard".into(),
        },
    });
    assert!(matches!(result, CommandResult::Accepted { .. }));
    let owner = engine.state().turn.current;
    (engine, owner)
}

fn suit_marks_of(tx: &Transaction, player: PlayerId) -> Vec<i32> {
    FALU_SUIT_MARKS
        .iter()
        .map(|def| {
            tx.state()
                .players
                .get(player)
                .and_then(|p| p.marks.iter().find(|(id, _)| *id == *def))
                .map(|(_, c)| *c)
                .unwrap_or(0)
        })
        .collect()
}

#[test]
fn committed_package_publishes_falu_game_start_trigger() {
    let pkg = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(pkg
        .trigger_index
        .iter()
        .any(|t| { t.skill_def_id == XINFU_FALU && t.event == "game_started" }));
}

#[test]
fn game_start_grants_four_marks() {
    let (engine, owner) = start();
    let mut tx = Transaction::begin(engine.state().clone());
    let effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    let mut effects = effects;
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(XINFU_FALU),
    });
    run_effects(&mut tx, effects).unwrap();
    assert_eq!(suit_marks_of(&tx, owner), [0, 0, 0, 0]);

    let effects = dispatch_for_events(
        &mut tx,
        &[DomainEvent::GameStarted { player_count: 2 }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap_or_else(|e| panic!("game-started dispatch error: {e:?}"));
    run_effects(&mut tx, effects).unwrap();

    assert_eq!(
        suit_marks_of(&tx, owner),
        [1, 1, 1, 1],
        "开局四花色标记各 1 枚"
    );
}
