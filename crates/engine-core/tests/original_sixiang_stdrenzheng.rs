//! Golden coverage for 标骆统「仁政」rewritten from legacy JavaScript.

use engine_api::{DomainEvent, PlayerId, SkillDefId};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{GameState, Phase};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const STDRENZHENG: u32 = 3_020_306_305;
const SOURCE: &str = include_str!("../../../content/original/ported/sixiang/stdrenzheng.skill");
const SIXIANG: &[u8] =
    include_bytes!("../../../content/original/ported/sixiang/sixiang.npack.json");

fn setup(extra_skill: &str, extra_def: u32) -> (Transaction, PlayerId, PlayerId, PlayerId) {
    let source = format!("{SOURCE}\n{extra_skill}");
    let package = content_compiler::compile_source_named("stdrenzheng-test", &source).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 3, 2_026_073_101, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    let players = tx.state().players.alive_by_seat();
    let current = players[0];
    let damage_source = players[1];
    let target = players[2];
    run_effects(
        &mut tx,
        vec![
            EffectRequest::AddSkill {
                player: current,
                skill_def: SkillDefId::new(STDRENZHENG),
            },
            EffectRequest::AddSkill {
                player: if extra_def == 4_200_000_002 {
                    damage_source
                } else {
                    target
                },
                skill_def: SkillDefId::new(extra_def),
            },
            EffectRequest::SetTurnState {
                current,
                phase: Phase::Play.to_u8(),
            },
        ],
    )
    .unwrap();
    (tx, current, damage_source, target)
}

fn assert_prevention_draws(
    mut tx: Transaction,
    current: PlayerId,
    source: PlayerId,
    target: PlayerId,
) {
    let hand_before = tx.state().hand_count(current);
    let hp_before = tx.state().players.get(target).unwrap().hp;
    let events = run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source,
            target,
            amount: 1,
            is_lose_hp: false,
            nature: rule_api::damage_nature::NORMAL,
        }],
    )
    .unwrap();
    assert_eq!(tx.state().players.get(target).unwrap().hp, hp_before);
    assert_eq!(tx.state().hand_count(current), hand_before + 1);
    assert!(events.iter().any(|event| matches!(
        event,
        DomainEvent::DamagePrevented {
            source: event_source,
            target: event_target,
            current: event_current,
        } if *event_source == source && *event_target == target && *event_current == current
    )));
}

#[test]
fn package_publishes_forced_damage_prevented_trigger_at_abi337() {
    let package = content_schema::ContentPackage::from_json_bytes(SIXIANG).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 562);
    assert_eq!(package.skill_index.len(), 181);
    let trigger = package
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == STDRENZHENG)
        .unwrap();
    assert_eq!(trigger.event, "damage_prevented");
    assert_eq!(trigger.filter, "owner_alive_event_player_alive");
    assert!(!trigger.optional);
}

#[test]
fn static_and_damage_before_prevention_both_make_current_player_draw() {
    let (tx, current, source, target) = setup(
        "skill pacifist(4200000002) locked { modifier damage_cancel override 1; }",
        4_200_000_002,
    );
    assert_prevention_draws(tx, current, source, target);

    let (tx, current, source, target) = setup(
        "skill shield(4200000003) locked {
            trigger damage_before owner_is_target {
                effect prevent_current_damage();
            }
        }",
        4_200_000_003,
    );
    assert_prevention_draws(tx, current, source, target);
}
