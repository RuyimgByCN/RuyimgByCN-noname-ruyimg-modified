//! Golden coverage for SP 孙尚香「返乡」(fanxiang).
//! Owner's prepare phase, once: +1 max HP, recover 1, lose 良助, gain 枭姬.

use engine_api::{
    Command, CommandKind, DomainEvent, ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId,
    TurnPhaseTag,
};
use engine_core::effect::run_effects;
use engine_core::state::GameState;
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use engine_core::Engine;
use rule_api::EffectRequest;

const FANXIANG: u32 = 3_943_157_500;
const LIANGZHU: u32 = 3_065_103_097;
const XIAOJI: u32 = 22;
const SOURCE: &str = include_str!("../../../../content/original/ported/sp/fanxiang.skill");
const LIANGZHU_SOURCE: &str = include_str!("../../../../content/original/ported/sp/liangzhu.skill");
const PACKAGE: &[u8] = include_bytes!("../../../../content/original/ported/sp/sp.npack.json");

fn setup() -> (GameState, PlayerId) {
    let compiled = content_compiler::compile_source_named(
        "fanxiang-test",
        &format!("{SOURCE}\n{LIANGZHU_SOURCE}"),
    )
    .unwrap();
    let mut engine = Engine::new();
    engine.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(20_260_815_01),
        kind: CommandKind::StartGame {
            player_count: 2,
            rng_seed: 20_260_815_01,
            ruleset: "identity-standard".into(),
        },
    });
    engine.load_package(&compiled).unwrap();
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
        skill_def: SkillDefId::new(LIANGZHU),
    });
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(FANXIANG),
    });
    run_effects(&mut tx, effects).unwrap();
    {
        let player = tx.state_mut().players.get_mut(owner).unwrap();
        player.max_hp = 3;
        player.hp = 3;
    }
    (tx.commit().state, owner)
}

fn fire_prepare(state: GameState, owner: PlayerId) -> GameState {
    let mut tx = Transaction::begin(state);
    let effects = dispatch_for_events(
        &mut tx,
        &[DomainEvent::TurnStateChanged {
            current: owner,
            phase: TurnPhaseTag::Prepare,
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap_or_else(|e| panic!("prepare dispatch error: {e:?}"));
    run_effects(&mut tx, effects).unwrap();
    tx.commit().state
}

fn has_def(state: &GameState, player: PlayerId, def: u32) -> bool {
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
                .is_some_and(|instance| instance.def.raw() == def)
        })
}

fn once_mark(state: &GameState, owner: PlayerId) -> i32 {
    state
        .players
        .get(owner)
        .and_then(|player| player.marks.iter().find(|(id, _)| *id == FANXIANG))
        .map(|(_, count)| *count)
        .unwrap_or(0)
}

#[test]
fn compiled_source_and_npack_publish_the_fanxiang_contract() {
    let compiled = content_compiler::compile_source_named("fanxiang", SOURCE).unwrap();
    assert!(compiled.skill_index.iter().any(|skill| {
        skill.skill_def_id == FANXIANG && skill.name == "fanxiang" && skill.locked
    }));
    let trigger = compiled
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == FANXIANG)
        .expect("fanxiang trigger");
    assert_eq!(trigger.event, "phase_prepare");
    assert_eq!(trigger.filter, "owner_is_target");
    assert!(!trigger.optional);

    let committed = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(committed
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == FANXIANG && skill.name == "fanxiang"));
    let pack_trigger = committed
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == FANXIANG)
        .expect("npack fanxiang trigger");
    assert_eq!(pack_trigger.event, "phase_prepare");
    assert_eq!(pack_trigger.filter, "owner_is_target");
    let actual = committed
        .modules
        .iter()
        .find(|module| module.name == compiled.modules[0].name)
        .expect("npack module for fanxiang");
    assert_eq!(actual.logic_hash, compiled.modules[0].logic_hash);
}

#[test]
fn prepare_gains_max_hp_recovers_swaps_liangzhu_for_xiaoji_once() {
    let (state, owner) = setup();
    assert!(has_def(&state, owner, LIANGZHU));
    assert!(!has_def(&state, owner, XIAOJI));
    let state = fire_prepare(state, owner);
    let player = state.players.get(owner).unwrap();
    assert_eq!(player.max_hp, 4);
    assert_eq!(player.hp, 4);
    assert_eq!(once_mark(&state, owner), 1);
    assert!(!has_def(&state, owner, LIANGZHU));
    assert!(has_def(&state, owner, XIAOJI));

    let state = fire_prepare(state, owner);
    let player = state.players.get(owner).unwrap();
    assert_eq!(player.max_hp, 4);
    assert_eq!(player.hp, 4);
    assert_eq!(once_mark(&state, owner), 1);
}
