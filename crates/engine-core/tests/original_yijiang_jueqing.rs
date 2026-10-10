//! Golden for 一将成名张春华「绝情」的伤害改失去体力语义。

use engine_api::{
    Command, CommandKind, CommandResult, DomainEvent, ExpectedRevision, IdempotencyKey, PlayerId,
    SkillDefId,
};
use engine_core::effect::{apply_effect, run_effects};
use engine_core::state::turn_flag;
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::EffectRequest;

const JUEQING: u32 = 3_481_582_878;
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/yijiang/yijiang.npack.json");

fn isolated_state(with_jueqing: bool) -> (Transaction, PlayerId, PlayerId) {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == JUEQING && skill.locked));

    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(JUEQING as u64),
            kind: CommandKind::StartGame {
                player_count: 2,
                rng_seed: 20_260_715,
                ruleset: "identity-standard".into(),
            },
        }),
        CommandResult::Accepted { .. }
    ));
    let owner = engine.state().turn.current;
    let target = engine
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
    if with_jueqing {
        effects.push(EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(JUEQING),
        });
    }
    run_effects(&mut tx, effects).unwrap();
    (tx, owner, target)
}

#[test]
fn damage_becomes_equal_hp_loss_without_damage_bookkeeping() {
    let (mut tx, owner, target) = isolated_state(true);
    let hp = tx.state().players.get(target).unwrap().hp;
    let events = apply_effect(
        &mut tx,
        EffectRequest::DealDamage {
            source: owner,
            target,
            amount: 2,
            is_lose_hp: false,
            nature: 1,
        },
    )
    .unwrap();

    assert!(matches!(
        events.as_slice(),
        [DomainEvent::DamageDealt {
            source,
            target: event_target,
            amount: 2,
            is_lose_hp: true,
            nature: 1,
        }] if *source == owner && *event_target == target
    ));
    assert_eq!(tx.state().players.get(target).unwrap().hp, hp - 2);
    assert!(!tx.state().damage_history.contains(&(owner, target)));
    assert_eq!(tx.state().turn.flag(turn_flag::DAMAGE_DEALT), 0);
    assert_eq!(tx.state().turn.damaged_player_count(), 0);

    let (mut ordinary, owner, target) = isolated_state(false);
    apply_effect(
        &mut ordinary,
        EffectRequest::DealDamage {
            source: owner,
            target,
            amount: 1,
            is_lose_hp: false,
            nature: 0,
        },
    )
    .unwrap();
    assert!(ordinary.state().damage_history.contains(&(owner, target)));
    assert_eq!(ordinary.state().turn.flag(turn_flag::DAMAGE_DEALT), 1);
    assert_eq!(ordinary.state().turn.damaged_player_count(), 1);
}
