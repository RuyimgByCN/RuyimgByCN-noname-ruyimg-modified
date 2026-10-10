//! Golden coverage for KEY 困愤 / 阵军 / 望尽 / 枢机续 rewritten from honest noops.

use engine_api::{
    ChoiceId, Command, CommandKind, CommandResult, DomainEvent, ExpectedRevision, IdempotencyKey,
    PlayerId, SkillDefId, TurnPhaseTag,
};
use engine_core::command::apply_command;
use engine_core::effect::{run_effects, EffectError};
use engine_core::state::GameState;
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use engine_core::Engine;
use rule_api::EffectRequest;

const YUZURU_KUNFEN: u32 = 2_231_368_330;
const LIYINGXIA_ZHENJUN: u32 = 3_556_873_289;
const TOMOYA_WANGJIN: u32 = 3_865_938_350;
const RUMI_SHUWU2: u32 = 3_596_925_719;

const KUNFEN_SRC: &str = include_str!("../../../content/original/ported/key/yuzuru_kunfen.skill");
const ZHENJUN_SRC: &str =
    include_str!("../../../content/original/ported/key/liyingxia_zhenjun.skill");
const WANGJIN_SRC: &str = include_str!("../../../content/original/ported/key/tomoya_wangjin.skill");
const SHUWU_SRC: &str = include_str!("../../../content/original/ported/key/rumi_shuwu2.skill");
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/key/key.npack.json");

fn setup(source: &str, name: &str, skill_def: u32) -> (GameState, PlayerId, PlayerId) {
    let compiled = content_compiler::compile_source_named(name, source).unwrap();
    let mut engine = Engine::new();
    engine.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(20_260_815_60),
        kind: CommandKind::StartGame {
            player_count: 2,
            rng_seed: 20_260_815_60,
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

fn fire_phase(state: GameState, current: PlayerId, phase: TurnPhaseTag) -> (GameState, bool) {
    let mut tx = Transaction::begin(state);
    match dispatch_for_events(
        &mut tx,
        &[DomainEvent::TurnStateChanged { current, phase }],
        &mut TriggerFireRecord::new(),
    ) {
        Err(EffectError::ChoiceRequested) => (tx.commit().state, true),
        Ok(effects) => {
            run_effects(&mut tx, effects).unwrap();
            (tx.commit().state, false)
        }
        Err(err) => panic!("phase dispatch error: {err:?}"),
    }
}

fn resolve(state: GameState, actor: PlayerId, selections: Vec<u32>, key: u64) -> GameState {
    let (state, pipe) = apply_command(
        state.clone(),
        Command {
            actor,
            expected_revision: ExpectedRevision(state.revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id: ChoiceId::new(state.resolution.pending_choice_id),
                selections,
                cancelled: false,
            },
        },
    );
    assert!(
        matches!(pipe.result, CommandResult::Accepted { .. }),
        "{:?}",
        pipe.result
    );
    state
}

#[test]
fn kunfen_source_and_npack_publish_end_lose_and_draw() {
    let compiled = content_compiler::compile_source_named("yuzuru_kunfen", KUNFEN_SRC).unwrap();
    let trigger = compiled
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == YUZURU_KUNFEN)
        .expect("kunfen trigger");
    assert_eq!(trigger.event, "phase_end");
    assert_eq!(trigger.filter, "owner_is_target");
    assert!(!trigger.optional);
    let pack = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(pack
        .trigger_index
        .iter()
        .any(|trigger| trigger.skill_def_id == YUZURU_KUNFEN && trigger.event == "phase_end"));
}

#[test]
fn kunfen_end_phase_loses_one_hp_and_draws_two() {
    let (mut state, owner, _) = setup(KUNFEN_SRC, "kunfen-test", YUZURU_KUNFEN);
    {
        let player = state.players.get_mut(owner).unwrap();
        player.max_hp = 4;
        player.hp = 3;
    }
    let before_hand = state.hand_count(owner);
    let before_hp = state.players.get(owner).unwrap().hp;
    let (state, requested) = fire_phase(state, owner, TurnPhaseTag::End);
    assert!(!requested);
    assert_eq!(state.players.get(owner).unwrap().hp, before_hp - 1);
    assert_eq!(state.hand_count(owner), before_hand + 2);
}

#[test]
fn zhenjun_source_and_npack_publish_end_draw() {
    let compiled = content_compiler::compile_source_named("liyingxia_zhenjun", ZHENJUN_SRC).unwrap();
    let trigger = compiled
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == LIYINGXIA_ZHENJUN)
        .expect("zhenjun trigger");
    assert_eq!(trigger.event, "phase_end");
    assert_eq!(trigger.filter, "owner_is_target");
    assert!(trigger.optional);
    let pack = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(pack.trigger_index.iter().any(|trigger| {
        trigger.skill_def_id == LIYINGXIA_ZHENJUN && trigger.event == "phase_end"
    }));
}

#[test]
fn zhenjun_end_phase_lets_chosen_player_draw_one() {
    let (state, owner, other) = setup(ZHENJUN_SRC, "zhenjun-test", LIYINGXIA_ZHENJUN);
    let other_hand = state.hand_count(other);
    let (state, requested) = fire_phase(state, owner, TurnPhaseTag::End);
    assert!(requested);
    let state = resolve(state, owner, vec![1], 2);
    let state = resolve(state, owner, vec![other.raw()], 3);
    assert_eq!(state.hand_count(other), other_hand + 1);
}

#[test]
fn wangjin_source_and_npack_publish_other_end_draw() {
    let compiled = content_compiler::compile_source_named("tomoya_wangjin", WANGJIN_SRC).unwrap();
    let trigger = compiled
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == TOMOYA_WANGJIN)
        .expect("wangjin trigger");
    assert_eq!(trigger.event, "phase_end");
    assert_eq!(trigger.filter, "owner_alive_not_target");
    assert!(trigger.optional);
    let pack = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(pack.trigger_index.iter().any(|trigger| {
        trigger.skill_def_id == TOMOYA_WANGJIN && trigger.filter == "owner_alive_not_target"
    }));
}

#[test]
fn wangjin_other_end_phase_draws_two() {
    let (state, owner, other) = setup(WANGJIN_SRC, "wangjin-test", TOMOYA_WANGJIN);
    let before = state.hand_count(owner);
    let (state, requested) = fire_phase(state, other, TurnPhaseTag::End);
    assert!(requested);
    let state = resolve(state, owner, vec![1], 4);
    assert_eq!(state.hand_count(owner), before + 2);
}

#[test]
fn shuwu_source_and_npack_publish_end_draw() {
    let compiled = content_compiler::compile_source_named("rumi_shuwu2", SHUWU_SRC).unwrap();
    let trigger = compiled
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == RUMI_SHUWU2)
        .expect("shuwu trigger");
    assert_eq!(trigger.event, "phase_end");
    assert_eq!(trigger.filter, "owner_is_target");
    assert!(!trigger.optional);
    let pack = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(pack
        .trigger_index
        .iter()
        .any(|trigger| trigger.skill_def_id == RUMI_SHUWU2 && trigger.event == "phase_end"));
}

#[test]
fn shuwu_end_phase_draws_one() {
    let (state, owner, _) = setup(SHUWU_SRC, "shuwu-test", RUMI_SHUWU2);
    let before = state.hand_count(owner);
    let (state, requested) = fire_phase(state, owner, TurnPhaseTag::End);
    assert!(!requested);
    assert_eq!(state.hand_count(owner), before + 1);
}
