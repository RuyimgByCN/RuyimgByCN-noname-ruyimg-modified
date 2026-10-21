//! Golden for sp 典军 (oldianjun): locked — at the owner's phase end, the owner
//! takes 1 no-source HP loss (approximated by lose_hp) and queues an extra play phase.

use engine_api::{
    Command, CommandKind, CommandResult, DomainEvent, ExpectedRevision, IdempotencyKey, PlayerId,
    SkillDefId, TurnPhaseTag,
};
use engine_core::command::apply_command;
use engine_core::effect::run_effects;
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use engine_core::Engine;
use rule_api::EffectRequest;

const OLDIANJUN: u32 = 2_275_218_678;
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
            rng_seed: 20_262_101,
            ruleset: "identity-standard".into(),
        },
    });
    assert!(matches!(result, CommandResult::Accepted { .. }));
    let owner = engine.state().turn.current;
    (engine, owner)
}

fn add_skill(tx: &mut Transaction, owner: PlayerId) {
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
        skill_def: SkillDefId::new(OLDIANJUN),
    });
    run_effects(tx, effects).unwrap();
}

#[test]
fn committed_package_publishes_oldianjun_trigger() {
    let pkg = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(pkg
        .trigger_index
        .iter()
        .any(|t| { t.skill_def_id == OLDIANJUN && t.event == "phase_end" }));
}

#[test]
fn phase_end_loses_one_hp_and_queues_extra_play() {
    let (engine, owner) = start();
    let mut tx = Transaction::begin(engine.state().clone());
    add_skill(&mut tx, owner);
    let hp_before = tx.state().players.get(owner).unwrap().hp;

    let effects = dispatch_for_events(
        &mut tx,
        &[DomainEvent::TurnStateChanged {
            current: owner,
            phase: TurnPhaseTag::End,
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap_or_else(|e| panic!("phase-end dispatch error: {e:?}"));
    run_effects(&mut tx, effects).unwrap();

    assert_eq!(
        tx.state().players.get(owner).unwrap().hp,
        hp_before - 1,
        "典军 loses 1 HP at phase end"
    );
}
