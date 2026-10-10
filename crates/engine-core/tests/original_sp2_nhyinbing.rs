//! Golden for SP2「阴兵」的杀伤害转体力流失与全局体力流失摸牌语义。

use engine_api::{
    CardId, Command, CommandKind, CommandResult, DomainEvent, ExpectedRevision, IdempotencyKey,
    PlayerId, SkillDefId,
};
use engine_core::effect::{apply_effect, run_effects};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use engine_core::Engine;
use rule_api::EffectRequest;

const NHYINBING: u32 = 2_701_847_677;
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/sp2/sp2.npack.json");

fn isolated_state() -> (Transaction, PlayerId, PlayerId, PlayerId) {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == NHYINBING && skill.locked));
    assert!(package.modifier_index.iter().any(|modifier| {
        modifier.skill_def_id == NHYINBING
            && modifier.target == "damage_as_hp_loss"
            && modifier.selector.as_deref() == Some("sha")
    }));

    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(NHYINBING as u64),
            kind: CommandKind::StartGame {
                player_count: 3,
                rng_seed: NHYINBING as u64,
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
        skill_def: SkillDefId::new(NHYINBING),
    });
    run_effects(&mut tx, effects).unwrap();
    (tx, owner, others[0], others[1])
}

#[test]
fn converts_only_the_owners_logical_sha_damage_and_draws_after_other_hp_loss() {
    let (mut tx, owner, other, third) = isolated_state();

    let ordinary = apply_effect(
        &mut tx,
        EffectRequest::DealDamage {
            source: owner,
            target: other,
            amount: 1,
            is_lose_hp: false,
            nature: 0,
        },
    )
    .unwrap();
    assert!(matches!(
        ordinary.as_slice(),
        [DomainEvent::DamageDealt {
            is_lose_hp: false,
            ..
        }]
    ));

    let hand = tx.state().hand_count(owner);
    let sha = apply_effect(
        &mut tx,
        EffectRequest::DealShaDamage {
            source: owner,
            target: third,
            card: CardId::NONE,
            amount: 1,
            nature: 0,
            card_flags: 0,
        },
    )
    .unwrap();
    assert!(matches!(
        sha.as_slice(),
        [DomainEvent::DamageDealt {
            is_lose_hp: true,
            ..
        }]
    ));
    let effects = dispatch_for_events(&mut tx, &sha, &mut TriggerFireRecord::new()).unwrap();
    run_effects(&mut tx, effects).unwrap();
    assert_eq!(tx.state().hand_count(owner), hand + 1);

    let incoming = apply_effect(
        &mut tx,
        EffectRequest::DealShaDamage {
            source: other,
            target: owner,
            card: CardId::NONE,
            amount: 1,
            nature: 0,
            card_flags: 0,
        },
    )
    .unwrap();
    assert!(matches!(
        incoming.as_slice(),
        [DomainEvent::DamageDealt {
            is_lose_hp: false,
            ..
        }]
    ));
}
