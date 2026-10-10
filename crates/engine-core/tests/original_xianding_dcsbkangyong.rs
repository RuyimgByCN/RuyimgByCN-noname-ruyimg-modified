//! Golden for 限定 谋典韦「亢勇」rewritten from the legacy JavaScript semantics.

use engine_api::{
    Command, CommandKind, CommandResult, DomainEvent, ExpectedRevision, IdempotencyKey, PlayerId,
    SkillDefId, TurnPhaseTag,
};
use engine_core::effect::run_effects;
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use engine_core::Engine;
use rule_api::EffectRequest;

const DCSBKANGYONG: u32 = 3_080_902_597;
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/xianding/xianding.npack.json");

fn isolated(hp: i32) -> (Transaction, PlayerId, PlayerId) {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(DCSBKANGYONG as u64),
            kind: CommandKind::StartGame {
                player_count: 2,
                rng_seed: DCSBKANGYONG as u64,
                ruleset: "identity-standard".into(),
            },
        }),
        CommandResult::Accepted { .. }
    ));

    let owner = engine.state().turn.current;
    let other = engine
        .state()
        .players
        .iter_live()
        .find(|player| player.id != owner)
        .unwrap()
        .id;
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
        skill_def: SkillDefId::new(DCSBKANGYONG),
    });
    run_effects(&mut tx, effects).unwrap();
    let player = tx.state_mut().players.get_mut(owner).unwrap();
    player.max_hp = 4;
    player.hp = hp;
    (tx, owner, other)
}

fn phase(player: PlayerId, phase: TurnPhaseTag) -> DomainEvent {
    DomainEvent::TurnStateChanged {
        current: player,
        phase,
    }
}

fn dispatch_and_apply(tx: &mut Transaction, event: DomainEvent) {
    let effects = dispatch_for_events(tx, &[event], &mut TriggerFireRecord::new()).unwrap();
    run_effects(tx, effects).unwrap();
}

fn mark_count(tx: &Transaction, owner: PlayerId) -> i32 {
    tx.state()
        .players
        .get(owner)
        .unwrap()
        .marks
        .iter()
        .find(|(mark, _)| *mark == DCSBKANGYONG)
        .map_or(0, |(_, count)| *count)
}

#[test]
fn committed_package_exposes_both_locked_turn_endpoints() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let contracts: Vec<_> = package
        .trigger_index
        .iter()
        .filter(|trigger| trigger.skill_def_id == DCSBKANGYONG)
        .map(|trigger| {
            (
                trigger.event.as_str(),
                trigger.filter.as_str(),
                trigger.optional,
            )
        })
        .collect();
    assert_eq!(
        contracts,
        vec![
            ("phase_begin", "owner_is_target_wounded", false),
            (
                "phase_end",
                "owner_is_target_mark_positive_3080902597",
                false,
            ),
        ]
    );
}

#[test]
fn turn_start_recovers_to_max_and_turn_end_loses_the_recorded_amount() {
    let (mut tx, owner, _) = isolated(2);
    dispatch_and_apply(&mut tx, phase(owner, TurnPhaseTag::Prepare));
    assert_eq!(tx.state().players.get(owner).unwrap().hp, 4);
    assert_eq!(mark_count(&tx, owner), 2);

    dispatch_and_apply(&mut tx, phase(owner, TurnPhaseTag::End));
    assert_eq!(tx.state().players.get(owner).unwrap().hp, 2);
    assert_eq!(mark_count(&tx, owner), 0);
}

#[test]
fn end_loss_stops_at_one_and_only_the_owners_end_phase_consumes_the_record() {
    let (mut tx, owner, other) = isolated(2);
    dispatch_and_apply(&mut tx, phase(owner, TurnPhaseTag::Prepare));
    tx.state_mut().players.get_mut(owner).unwrap().hp = 2;

    dispatch_and_apply(&mut tx, phase(other, TurnPhaseTag::End));
    assert_eq!(tx.state().players.get(owner).unwrap().hp, 2);
    assert_eq!(mark_count(&tx, owner), 2);

    dispatch_and_apply(&mut tx, phase(owner, TurnPhaseTag::End));
    assert_eq!(tx.state().players.get(owner).unwrap().hp, 1);
    assert_eq!(mark_count(&tx, owner), 0);
}

#[test]
fn healthy_turn_start_does_not_record_a_debt() {
    let (mut tx, owner, _) = isolated(4);
    let effects = dispatch_for_events(
        &mut tx,
        &[phase(owner, TurnPhaseTag::Prepare)],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    assert!(effects.is_empty());
    assert_eq!(mark_count(&tx, owner), 0);
}
