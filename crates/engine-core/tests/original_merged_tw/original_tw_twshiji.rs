//! Golden coverage for TW「势击」(twshiji): prepare-phase draw 2.

use engine_api::{DomainEvent, PlayerId, SkillDefId, TurnPhaseTag};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::GameState;
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const TWSHIJI: u32 = 2_278_923_284;
const SOURCE: &str = include_str!("../../../../content/original/ported/tw/twshiji.skill");
const PACKAGE: &[u8] = include_bytes!("../../../../content/original/ported/tw/tw.npack.json");

fn setup() -> (GameState, PlayerId) {
    let compiled = content_compiler::compile_source_named("twshiji-test", SOURCE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 2, 20_260_815_82, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content)
        .load_package(&compiled)
        .unwrap();
    let owner = tx.state().turn.current;
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    for player in tx.state().players.alive_by_seat() {
        effects.push(EffectRequest::DiscardFromHand {
            player,
            count: u32::MAX,
        });
    }
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(TWSHIJI),
    });
    run_effects(&mut tx, effects).unwrap();
    (tx.commit().state, owner)
}

#[test]
fn compiled_source_and_npack_publish_prepare_draw() {
    let compiled = content_compiler::compile_source_named("twshiji", SOURCE).unwrap();
    let trigger = compiled
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == TWSHIJI)
        .unwrap();
    assert_eq!(trigger.event, "phase_prepare");
    assert!(!trigger.optional);
    let pack = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(pack
        .trigger_index
        .iter()
        .any(|trigger| trigger.skill_def_id == TWSHIJI && trigger.event == "phase_prepare"));
}

#[test]
fn prepare_draws_two() {
    let (state, owner) = setup();
    let before = state.hand_count(owner);
    let mut tx = Transaction::begin(state);
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
    assert_eq!(tx.state().hand_count(owner), before + 2);
}
