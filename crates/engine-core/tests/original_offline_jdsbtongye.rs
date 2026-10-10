//! Golden for offline 通夜 (jdsbtongye): locked — at game start (draw pile never
//! shuffled yet) the owner gains 英姿 and 固政.

use engine_api::{
    Command, CommandKind, CommandResult, DomainEvent, ExpectedRevision, IdempotencyKey, PlayerId,
    SkillDefId,
};
use engine_core::effect::run_effects;
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use engine_core::Engine;
use rule_api::EffectRequest;

const JDSBTONGYE: u32 = 2_312_805_637;
const YINGZI: u32 = 20;
const GUZHENG: u32 = 4_159_731_006;
const OFFLINE: &[u8] = include_bytes!("../../../content/original/ported/offline/offline.npack.json");
const SHENHUA: &[u8] = include_bytes!("../../../content/original/ported/shenhua/shenhua.npack.json");

fn start() -> (Engine, PlayerId) {
    let mut engine = Engine::new();
    for bytes in [OFFLINE, SHENHUA] {
        let pkg = content_schema::ContentPackage::from_json_bytes(bytes).unwrap();
        engine.load_package(&pkg).unwrap();
    }
    let result = engine.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(0),
        kind: CommandKind::StartGame {
            player_count: 2,
            rng_seed: 20_262_301,
            ruleset: "identity-standard".into(),
        },
    });
    assert!(matches!(result, CommandResult::Accepted { .. }));
    let owner = engine.state().turn.current;
    (engine, owner)
}

fn has_skill_def(tx: &Transaction, player: PlayerId, def: u32) -> bool {
    tx.state()
        .skills
        .sorted_live()
        .iter()
        .any(|skill| skill.owner == player && skill.def.raw() == def)
}

#[test]
fn committed_package_publishes_jdsbtongye_trigger() {
    let pkg = content_schema::ContentPackage::from_json_bytes(OFFLINE).unwrap();
    assert!(pkg.trigger_index.iter().any(|t| {
        t.skill_def_id == JDSBTONGYE && t.event == "game_started"
    }));
}

#[test]
fn game_start_grants_yingzi_and_guzheng() {
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
        skill_def: SkillDefId::new(JDSBTONGYE),
    });
    run_effects(&mut tx, effects).unwrap();
    assert!(!has_skill_def(&tx, owner, YINGZI));

    let effects = dispatch_for_events(
        &mut tx,
        &[DomainEvent::GameStarted { player_count: 2 }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap_or_else(|e| panic!("game-started dispatch error: {e:?}"));
    run_effects(&mut tx, effects).unwrap();

    assert!(has_skill_def(&tx, owner, YINGZI), "英姿 granted");
    assert!(has_skill_def(&tx, owner, GUZHENG), "固政 granted");
}
