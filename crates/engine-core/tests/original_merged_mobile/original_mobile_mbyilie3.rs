//! Golden for MOBILE「义烈」结束阶段子技能：消费全部“烈”并结算摸牌与失去体力。

use engine_api::{
    Command, CommandKind, CommandResult, DomainEvent, ExpectedRevision, IdempotencyKey, PlayerId,
    SkillDefId, TurnPhaseTag,
};
use engine_core::effect::run_effects;
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use engine_core::Engine;
use rule_api::EffectRequest;

const MBYILIE: u32 = 3_247_391_666;
const MBYILIE3: u32 = 2_964_107_248;
const PACKAGE: &[u8] =
    include_bytes!("../../../../content/original/ported/mobile/mobile.npack.json");

fn setup(mark_count: i32) -> (Transaction, PlayerId) {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package.trigger_index.iter().any(|trigger| {
        trigger.skill_def_id == MBYILIE3
            && trigger.event == "phase_end"
            && trigger.filter == "owner_mark_positive_3247391666"
            && !trigger.optional
    }));

    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(MBYILIE3 as u64),
            kind: CommandKind::StartGame {
                player_count: 2,
                rng_seed: MBYILIE3 as u64,
                ruleset: "identity-standard".into(),
            },
        }),
        CommandResult::Accepted { .. }
    ));
    let owner = engine.state().turn.current;
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
        skill_def: SkillDefId::new(MBYILIE3),
    });
    if mark_count > 0 {
        effects.push(EffectRequest::AddMark {
            player: owner,
            mark: engine_api::MarkDefId::new(MBYILIE),
            delta: mark_count,
        });
    }
    run_effects(&mut tx, effects).unwrap();
    let player = tx.state_mut().players.get_mut(owner).unwrap();
    player.hp = 4;
    player.max_hp = 4;
    (tx, owner)
}

fn phase_end(owner: PlayerId) -> DomainEvent {
    DomainEvent::TurnStateChanged {
        current: owner,
        phase: TurnPhaseTag::End,
    }
}

fn mark_count(tx: &Transaction, owner: PlayerId) -> i32 {
    tx.state()
        .players
        .get(owner)
        .unwrap()
        .marks
        .iter()
        .find(|(mark, _)| *mark == MBYILIE)
        .map_or(0, |(_, count)| *count)
}

#[test]
fn positive_mark_draws_then_loses_equal_hp_and_clears_the_mark() {
    let (mut tx, owner) = setup(2);
    let hand = tx.state().hand_count(owner);

    let effects =
        dispatch_for_events(&mut tx, &[phase_end(owner)], &mut TriggerFireRecord::new()).unwrap();
    run_effects(&mut tx, effects).unwrap();

    assert_eq!(tx.state().hand_count(owner), hand + 1);
    assert_eq!(tx.state().players.get(owner).unwrap().hp, 2);
    assert_eq!(mark_count(&tx, owner), 0);
}

#[test]
fn zero_mark_does_not_fire() {
    let (mut tx, owner) = setup(0);
    let hand = tx.state().hand_count(owner);
    let hp = tx.state().players.get(owner).unwrap().hp;

    assert!(
        dispatch_for_events(&mut tx, &[phase_end(owner)], &mut TriggerFireRecord::new(),)
            .unwrap()
            .is_empty()
    );
    assert_eq!(tx.state().hand_count(owner), hand);
    assert_eq!(tx.state().players.get(owner).unwrap().hp, hp);
}
