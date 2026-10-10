//! Golden coverage for SP 诸葛诞「举义」and its SHENHUA dependency.

use engine_api::{DomainEvent, PlayerId, SkillDefId, TurnPhaseTag};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{GameState, Phase};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const ZHUGEDAN: u32 = 2_327_484_329;
const GONGAO: u32 = 2_416_186_304;
const JUYI: u32 = 2_383_670_146;
const BENGHUAI: u32 = 3_365_741_831;
const WEIZHONG: u32 = 2_998_881_378;
const SOURCE: &str = include_str!("../../../content/original/ported/sp/juyi.skill");
const SP_PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/sp/sp.npack.json");
const SHENHUA_PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/shenhua/shenhua.npack.json");

fn has_skill_def(state: &GameState, player: PlayerId, skill_def: u32) -> bool {
    state
        .players
        .get(player)
        .unwrap()
        .skills
        .iter()
        .any(|skill| {
            state
                .skills
                .get(*skill)
                .is_some_and(|skill| skill.def.raw() == skill_def)
        })
}

fn enter_prepare(tx: &mut Transaction, owner: PlayerId) {
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Prepare;
    let effects = dispatch_for_events(
        tx,
        &[DomainEvent::TurnStateChanged {
            current: owner,
            phase: TurnPhaseTag::Prepare,
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    run_effects(tx, effects).unwrap();
}

fn setup() -> (Transaction, PlayerId) {
    let source =
        format!("{SOURCE}\nskill benghuai({BENGHUAI}) {{}}\nskill weizhong({WEIZHONG}) {{}}\n");
    let package = content_compiler::compile_source_named("juyi-test", &source).unwrap();
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package).unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 4, 20_260_723, "identity-standard").unwrap();
    let owner = tx.state().players.alive_by_seat()[0];
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(JUYI),
    });
    run_effects(&mut tx, effects).unwrap();
    (tx, owner)
}

#[test]
fn committed_package_exposes_juyi_and_complete_zhugedan_closure() {
    let shenhua = content_schema::ContentPackage::from_json_bytes(SHENHUA_PACKAGE).unwrap();
    let sp = content_schema::ContentPackage::from_json_bytes(SP_PACKAGE).unwrap();
    assert!(sp.manifest.dependencies.iter().any(|dependency| {
        dependency.namespace == "official"
            && dependency.package_id == "shenhua"
            && dependency.version == "0.1.0"
    }));
    assert!(sp.trigger_index.iter().any(|trigger| {
        trigger.skill_def_id == JUYI
            && trigger.event == "phase_prepare"
            && trigger.filter == "owner_is_target"
            && !trigger.optional
    }));
    let character = sp
        .character_index
        .iter()
        .find(|character| character.id == ZHUGEDAN)
        .unwrap();
    assert_eq!(character.name_key, "zhugedan");
    assert_eq!(character.sex, "male");
    assert_eq!(character.group, "wei");
    assert_eq!(character.max_hp, 4);
    assert_eq!(character.skills, vec![GONGAO, JUYI]);

    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&shenhua).unwrap();
    std::sync::Arc::make_mut(&mut state.content).load_package(&sp).unwrap();
}

#[test]
fn juyi_requires_strictly_more_max_hp_and_awakens_only_once() {
    let (mut tx, owner) = setup();
    tx.state_mut().players.get_mut(owner).unwrap().max_hp = 4;
    tx.state_mut().players.get_mut(owner).unwrap().hp = 4;
    let hand_before = tx.state().hand_count(owner);

    enter_prepare(&mut tx, owner);
    assert_eq!(tx.state().hand_count(owner), hand_before);
    assert!(has_skill_def(tx.state(), owner, JUYI));

    tx.state_mut().players.get_mut(owner).unwrap().max_hp = 5;
    tx.state_mut().players.get_mut(owner).unwrap().hp = 5;
    enter_prepare(&mut tx, owner);
    assert_eq!(tx.state().hand_count(owner), hand_before + 5);
    assert!(!has_skill_def(tx.state(), owner, JUYI));
    assert!(has_skill_def(tx.state(), owner, BENGHUAI));
    assert!(has_skill_def(tx.state(), owner, WEIZHONG));

    enter_prepare(&mut tx, owner);
    assert_eq!(tx.state().hand_count(owner), hand_before + 5);
}
