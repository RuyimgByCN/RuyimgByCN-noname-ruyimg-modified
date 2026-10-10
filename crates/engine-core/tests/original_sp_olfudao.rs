//! Golden coverage for SP OL 丁尚涴「抚悼」(olfudao).
//! Game start: owner draws 3.

use engine_api::{DomainEvent, PlayerId, SkillDefId};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::GameState;
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const OLFUDAO: u32 = 2_837_007_198;
const SOURCE: &str = include_str!("../../../content/original/ported/sp/olfudao.skill");
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/sp/sp.npack.json");

fn setup() -> (GameState, PlayerId) {
    let compiled = content_compiler::compile_source_named("olfudao-test", SOURCE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 2, 20_260_815_11, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&compiled).unwrap();
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
        skill_def: SkillDefId::new(OLFUDAO),
    });
    run_effects(&mut tx, effects).unwrap();
    (tx.commit().state, owner)
}

#[test]
fn compiled_source_and_npack_publish_the_game_start_draw() {
    let compiled = content_compiler::compile_source_named("olfudao", SOURCE).unwrap();
    assert!(compiled
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == OLFUDAO && skill.name == "olfudao" && !skill.locked));
    let trigger = compiled
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == OLFUDAO)
        .expect("olfudao trigger");
    assert_eq!(trigger.event, "game_started");
    assert_eq!(trigger.filter, "owner_alive");
    assert!(!trigger.optional);

    let committed = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let pack = committed
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == OLFUDAO)
        .expect("npack olfudao trigger");
    assert_eq!(pack.event, "game_started");
    assert_eq!(pack.filter, "owner_alive");
    let actual = committed
        .modules
        .iter()
        .find(|module| module.name == compiled.modules[0].name)
        .expect("npack module for olfudao");
    assert_eq!(actual.logic_hash, compiled.modules[0].logic_hash);
}

#[test]
fn game_start_draws_three() {
    let (state, owner) = setup();
    let before = state.hand_count(owner);
    let mut tx = Transaction::begin(state);
    let effects = dispatch_for_events(
        &mut tx,
        &[DomainEvent::GameStarted { player_count: 2 }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    run_effects(&mut tx, effects).unwrap();
    assert_eq!(tx.state().hand_count(owner), before + 3);
}
