//! Golden coverage for JSRG 皆兵 / 悟道 rewritten from honest noops.

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

const JSRGJIEBING: u32 = 3_843_907_824;
const JSRGBAOWEI: u32 = 3_914_522_334;
const JSRGWUDAO: u32 = 2_550_512_310;
const JSRGJINGLEI: u32 = 2_695_101_489;

const JIEBING_SRC: &str =
    include_str!("../../../../content/original/ported/jsrg/jsrgjiebing.skill");
const WUDAO_SRC: &str = include_str!("../../../../content/original/ported/jsrg/jsrgwudao.skill");
const PACKAGE: &[u8] = include_bytes!("../../../../content/original/ported/jsrg/jsrg.npack.json");

fn setup(source: &str, name: &str, skill_def: u32) -> (GameState, PlayerId, PlayerId) {
    let compiled = content_compiler::compile_source_named(name, source).unwrap();
    let mut engine = Engine::new();
    engine.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(20_260_815_51),
        kind: CommandKind::StartGame {
            player_count: 2,
            rng_seed: 20_260_815_51,
            ruleset: "identity-standard".into(),
        },
    });
    engine.load_package(&compiled).unwrap();
    let owner = engine.state().turn.current;
    let other = engine
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
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(skill_def),
    });
    run_effects(&mut tx, effects).unwrap();
    (tx.commit().state, owner, other)
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

fn once_mark(state: &GameState, owner: PlayerId, skill: u32) -> i32 {
    state
        .players
        .get(owner)
        .and_then(|player| player.marks.iter().find(|(id, _)| *id == skill))
        .map(|(_, count)| *count)
        .unwrap_or(0)
}

#[test]
fn jiebing_source_and_npack_publish_prepare_awaken() {
    let compiled = content_compiler::compile_source_named("jsrgjiebing", JIEBING_SRC).unwrap();
    let trigger = compiled
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == JSRGJIEBING)
        .expect("jiebing trigger");
    assert_eq!(trigger.event, "phase_prepare");
    assert_eq!(trigger.filter, "owner_is_target");
    assert!(!trigger.optional);
    let pack = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(pack.trigger_index.iter().any(|trigger| {
        trigger.skill_def_id == JSRGJIEBING && trigger.event == "phase_prepare"
    }));
}

#[test]
fn jiebing_prepare_gains_two_max_hp_and_baowei_once() {
    let (mut state, owner, _) = setup(JIEBING_SRC, "jiebing-test", JSRGJIEBING);
    {
        let player = state.players.get_mut(owner).unwrap();
        player.max_hp = 3;
        player.hp = 3;
    }
    assert!(!has_def(&state, owner, JSRGBAOWEI));
    let state = fire_prepare(state, owner);
    let player = state.players.get(owner).unwrap();
    assert_eq!(player.max_hp, 5);
    assert_eq!(player.hp, 5);
    assert_eq!(once_mark(&state, owner, JSRGJIEBING), 1);
    assert!(has_def(&state, owner, JSRGBAOWEI));

    let state = fire_prepare(state, owner);
    let player = state.players.get(owner).unwrap();
    assert_eq!(player.max_hp, 5);
    assert_eq!(once_mark(&state, owner, JSRGJIEBING), 1);
}

#[test]
fn wudao_source_and_npack_publish_dying_awaken() {
    let compiled = content_compiler::compile_source_named("jsrgwudao", WUDAO_SRC).unwrap();
    let trigger = compiled
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == JSRGWUDAO)
        .expect("wudao trigger");
    assert_eq!(trigger.event, "dying");
    assert_eq!(trigger.filter, "owner_alive");
    assert!(!trigger.optional);
    let pack = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(pack
        .trigger_index
        .iter()
        .any(|trigger| { trigger.skill_def_id == JSRGWUDAO && trigger.event == "dying" }));
}

#[test]
fn wudao_on_dying_gains_max_hp_and_jinglei_once() {
    let (mut state, owner, other) = setup(WUDAO_SRC, "wudao-test", JSRGWUDAO);
    {
        let player = state.players.get_mut(owner).unwrap();
        player.max_hp = 3;
        player.hp = 2;
    }
    assert!(!has_def(&state, owner, JSRGJINGLEI));
    let mut tx = Transaction::begin(state);
    let effects = dispatch_for_events(
        &mut tx,
        &[DomainEvent::EnteredDying {
            player: other,
            source: owner,
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap_or_else(|e| panic!("dying dispatch error: {e:?}"));
    run_effects(&mut tx, effects).unwrap();
    let state = tx.commit().state;
    let player = state.players.get(owner).unwrap();
    assert_eq!(player.max_hp, 4);
    assert_eq!(player.hp, 3);
    assert_eq!(once_mark(&state, owner, JSRGWUDAO), 1);
    assert!(has_def(&state, owner, JSRGJINGLEI));

    let mut tx = Transaction::begin(state);
    let effects = dispatch_for_events(
        &mut tx,
        &[DomainEvent::EnteredDying {
            player: other,
            source: owner,
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap_or_else(|e| panic!("second dying dispatch error: {e:?}"));
    run_effects(&mut tx, effects).unwrap();
    let state = tx.commit().state;
    assert_eq!(state.players.get(owner).unwrap().max_hp, 4);
    assert_eq!(once_mark(&state, owner, JSRGWUDAO), 1);
}
