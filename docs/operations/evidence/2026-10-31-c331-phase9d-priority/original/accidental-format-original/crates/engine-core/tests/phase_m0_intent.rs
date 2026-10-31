//! Declarative DSL modifiers operate on typed intents before mutation.

use content_compiler::compile_source;
use engine_api::{PlayerId, SkillDefId};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::GameState;
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const PACK: &str = r#"
skill forceful(105) locked {
  modifier damage_amount add 2;
}

skill pacifist(106) locked {
  modifier damage_cancel override 1;
}
"#;

fn setup(skill_def: u32) -> (Transaction, PlayerId, PlayerId) {
    let package = compile_source(PACK).expect("compile intent package");
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content)
        .load_package(&package)
        .expect("load package");
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 2, 51, "identity-standard").expect("bootstrap");
    let source = PlayerId::new(1);
    let target = PlayerId::new(2);
    run_effects(
        &mut tx,
        vec![
            EffectRequest::AddSkill {
                player: source,
                skill_def: SkillDefId::new(skill_def),
            },
            EffectRequest::ChangeMaxHp {
                player: target,
                delta: 5,
            },
            EffectRequest::RecoverHp {
                source: target,
                player: target,
                amount: 5,
            },
        ],
    )
    .expect("prepare players");
    (tx, source, target)
}

#[test]
fn damage_amount_modifier_changes_intent_before_hp_mutation() {
    let (mut tx, source, target) = setup(105);
    let before = tx.state().players.get(target).unwrap().hp;
    run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source,
            target,
            amount: 1,
            is_lose_hp: false,
            nature: 0,
        }],
    )
    .expect("damage");
    assert_eq!(tx.state().players.get(target).unwrap().hp, before - 3);
}

#[test]
fn damage_cancel_modifier_vetoes_mutation_and_emits_prevention() {
    let (mut tx, source, target) = setup(106);
    let before = tx.state().players.get(target).unwrap().hp;
    let events = run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source,
            target,
            amount: 1,
            is_lose_hp: false,
            nature: 0,
        }],
    )
    .expect("cancelled damage");
    assert_eq!(tx.state().players.get(target).unwrap().hp, before);
    assert!(matches!(
        events.as_slice(),
        [engine_api::DomainEvent::DamagePrevented {
            source: event_source,
            target: event_target,
            current,
        }] if *event_source == source && *event_target == target && current.is_some()
    ));
}
