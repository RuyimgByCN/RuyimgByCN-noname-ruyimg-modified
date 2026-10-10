//! Golden for SP2「刚直」的双向伤害转体力流失语义。

use engine_api::{
    Command, CommandKind, CommandResult, DomainEvent, ExpectedRevision, IdempotencyKey, PlayerId,
    SkillDefId,
};
use engine_core::effect::{apply_effect, run_effects};
use engine_core::intent::{resolve_damage_intent, DamageIntent};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::EffectRequest;

const GANGZHI: u32 = 3_573_807_347;
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/sp2/sp2.npack.json");

fn isolated_state() -> (Transaction, PlayerId, PlayerId, PlayerId) {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == GANGZHI && skill.locked));

    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(GANGZHI as u64),
            kind: CommandKind::StartGame {
                player_count: 3,
                rng_seed: GANGZHI as u64,
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
        skill_def: SkillDefId::new(GANGZHI),
    });
    run_effects(&mut tx, effects).unwrap();
    (tx, owner, others[0], others[1])
}

fn resolved(tx: &Transaction, source: PlayerId, target: PlayerId) -> DamageIntent {
    resolve_damage_intent(
        tx.state(),
        DamageIntent {
            source,
            target,
            amount: 1,
            is_lose_hp: false,
            nature: 0,
            cancelled: false,
        },
    )
}

#[test]
fn converts_only_damage_between_the_owner_and_another_living_player() {
    let (mut tx, owner, other, third) = isolated_state();
    assert!(resolved(&tx, owner, other).is_lose_hp);
    assert!(resolved(&tx, other, owner).is_lose_hp);
    assert!(!resolved(&tx, owner, owner).is_lose_hp);
    assert!(!resolved(&tx, PlayerId::NONE, owner).is_lose_hp);
    assert!(!resolved(&tx, other, third).is_lose_hp);

    let hp = tx.state().players.get(other).unwrap().hp;
    let events = apply_effect(
        &mut tx,
        EffectRequest::DealDamage {
            source: owner,
            target: other,
            amount: 2,
            is_lose_hp: false,
            nature: 1,
        },
    )
    .unwrap();
    assert!(matches!(
        events.as_slice(),
        [DomainEvent::DamageDealt {
            is_lose_hp: true,
            ..
        }]
    ));
    assert_eq!(tx.state().players.get(other).unwrap().hp, hp - 2);
    assert!(!tx.state().damage_history.contains(&(owner, other)));

    tx.state_mut().players.get_mut(other).unwrap().alive = false;
    assert!(!resolved(&tx, other, owner).is_lose_hp);
}
