//! Golden coverage for MOBILE 来敏「来寿」rewritten from legacy JavaScript semantics.

use engine_api::{DomainEvent, PlayerId, SkillDefId, TurnPhaseTag};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{GameState, Phase};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const LAISHOU: u32 = 2_235_144_457;
const LAIMIN: u32 = 4_094_388_327;
const SOURCE: &str = include_str!("../../../content/original/ported/mobile/laishou.skill");
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/mobile/mobile.npack.json");

fn setup() -> (Transaction, PlayerId, PlayerId) {
    let package = content_compiler::compile_source_named("laishou-test", SOURCE).unwrap();
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package).unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 2, 2_026_073_302, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let owner = players[0];
    let source = players[1];
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(LAISHOU),
    });
    run_effects(&mut tx, effects).unwrap();
    let player = tx.state_mut().players.get_mut(owner).unwrap();
    player.hp = 3;
    player.max_hp = 3;
    (tx, owner, source)
}

fn damage(tx: &mut Transaction, source: PlayerId, target: PlayerId, amount: u32) {
    run_effects(
        tx,
        vec![EffectRequest::DealDamage {
            source,
            target,
            amount,
            is_lose_hp: false,
            nature: rule_api::damage_nature::NORMAL,
        }],
    )
    .unwrap();
}

#[test]
fn package_publishes_laimin_after_luanqun_closes_character() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == LAISHOU && skill.locked));
    assert_eq!(
        package
            .character_index
            .iter()
            .find(|character| character.id == LAIMIN)
            .unwrap()
            .skills,
        [LAISHOU, 3_119_764_480]
    );
}

#[test]
fn lethal_damage_grows_max_hp_and_is_prevented_then_prepare_kills_at_nine() {
    let (mut tx, owner, source) = setup();
    damage(&mut tx, source, owner, 2);
    assert_eq!(tx.state().players.get(owner).unwrap().hp, 1);
    assert_eq!(tx.state().players.get(owner).unwrap().max_hp, 3);

    let player = tx.state_mut().players.get_mut(owner).unwrap();
    player.hp = 3;
    player.max_hp = 3;
    damage(&mut tx, source, owner, 3);
    assert_eq!(tx.state().players.get(owner).unwrap().hp, 3);
    assert_eq!(tx.state().players.get(owner).unwrap().max_hp, 6);

    damage(&mut tx, source, owner, 4);
    assert_eq!(tx.state().players.get(owner).unwrap().hp, 3);
    assert_eq!(tx.state().players.get(owner).unwrap().max_hp, 10);

    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Prepare;
    let effects = dispatch_for_events(
        &mut tx,
        &[DomainEvent::TurnStateChanged {
            current: owner,
            phase: TurnPhaseTag::Prepare,
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    run_effects(&mut tx, effects).unwrap();
    assert!(!tx.state().players.get(owner).unwrap().alive);
}
