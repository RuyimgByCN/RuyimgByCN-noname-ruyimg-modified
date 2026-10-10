//! Golden coverage for KEY 一番宝物 / 伴音 / 制裸 / 托老 rewritten from honest noops.

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

const YUI_TAKARAMONO: u32 = 3_352_203_313;
const YUI_YINHANG: u32 = 3_807_513_032;
const SHIORIMIYUKI_BANYIN: u32 = 3_141_719_441;
const MIKI_ZHILUO: u32 = 2_539_237_675;
const HIROTO_TUOLAO: u32 = 2_920_912_067;
const HIROTO_HUYU: u32 = 2_586_716_910;
const HIROTO_ZONGLVE: u32 = 3_175_747_508;

const TAKARAMONO_SRC: &str =
    include_str!("../../../content/original/ported/key/yui_takaramono.skill");
const BANYIN_SRC: &str =
    include_str!("../../../content/original/ported/key/shiorimiyuki_banyin.skill");
const ZHILUO_SRC: &str = include_str!("../../../content/original/ported/key/miki_zhiluo.skill");
const TUOLAO_SRC: &str = include_str!("../../../content/original/ported/key/hiroto_tuolao.skill");
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/key/key.npack.json");

fn setup(source: &str, name: &str, skill_def: u32, extras: &[u32]) -> (GameState, PlayerId, PlayerId) {
    let compiled = content_compiler::compile_source_named(name, source).unwrap();
    let mut engine = Engine::new();
    engine.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(20_260_815_30),
        kind: CommandKind::StartGame {
            player_count: 2,
            rng_seed: 20_260_815_30,
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
    for extra in extras {
        effects.push(EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(*extra),
        });
    }
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

fn has_def(state: &GameState, player: PlayerId, def: u32) -> bool {
    state
        .players
        .get(player)
        .unwrap()
        .skills
        .iter()
        .any(|skill| state.skills.get(*skill).is_some_and(|instance| instance.def.raw() == def))
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
fn takaramono_source_and_npack_publish_prepare_awaken() {
    let compiled = content_compiler::compile_source_named("yui_takaramono", TAKARAMONO_SRC).unwrap();
    let trigger = compiled
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == YUI_TAKARAMONO)
        .expect("takaramono trigger");
    assert_eq!(trigger.event, "phase_prepare");
    assert_eq!(trigger.filter, "owner_is_target");
    assert!(!trigger.optional);
    let pack = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(pack.trigger_index.iter().any(|trigger| {
        trigger.skill_def_id == YUI_TAKARAMONO && trigger.event == "phase_prepare"
    }));
}

#[test]
fn takaramono_prepare_gains_max_hp_and_yinhang_once() {
    let (mut state, owner, _) = setup(TAKARAMONO_SRC, "takaramono-test", YUI_TAKARAMONO, &[]);
    {
        let player = state.players.get_mut(owner).unwrap();
        player.max_hp = 3;
        player.hp = 3;
    }
    assert!(!has_def(&state, owner, YUI_YINHANG));
    let (state, requested) = fire_phase(state, owner, TurnPhaseTag::Prepare);
    assert!(!requested);
    let player = state.players.get(owner).unwrap();
    assert_eq!(player.max_hp, 4);
    assert_eq!(player.hp, 4);
    assert_eq!(once_mark(&state, owner, YUI_TAKARAMONO), 1);
    assert!(has_def(&state, owner, YUI_YINHANG));

    let (state, _) = fire_phase(state, owner, TurnPhaseTag::Prepare);
    let player = state.players.get(owner).unwrap();
    assert_eq!(player.max_hp, 4);
    assert_eq!(once_mark(&state, owner, YUI_TAKARAMONO), 1);
}

#[test]
fn banyin_source_and_npack_publish_damage_and_recover() {
    let compiled =
        content_compiler::compile_source_named("shiorimiyuki_banyin", BANYIN_SRC).unwrap();
    let events: Vec<_> = compiled
        .trigger_index
        .iter()
        .filter(|trigger| trigger.skill_def_id == SHIORIMIYUKI_BANYIN)
        .map(|trigger| trigger.event.as_str())
        .collect();
    assert!(events.contains(&"damage"));
    assert!(events.contains(&"hp_recovered"));
    assert!(compiled.trigger_index.iter().all(|trigger| {
        trigger.skill_def_id != SHIORIMIYUKI_BANYIN || trigger.optional
    }));
    let pack = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(pack
        .trigger_index
        .iter()
        .any(|trigger| trigger.skill_def_id == SHIORIMIYUKI_BANYIN && trigger.event == "damage"));
    assert!(pack.trigger_index.iter().any(|trigger| {
        trigger.skill_def_id == SHIORIMIYUKI_BANYIN && trigger.event == "hp_recovered"
    }));
}

#[test]
fn banyin_after_damage_recovers_other() {
    let (mut state, owner, other) = setup(BANYIN_SRC, "banyin-test", SHIORIMIYUKI_BANYIN, &[]);
    {
        let player = state.players.get_mut(other).unwrap();
        player.hp = player.max_hp as i32 - 1;
    }
    let other_hp = state.players.get(other).unwrap().hp;
    let mut tx = Transaction::begin(state);
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::DealDamage {
                source: other,
                target: owner,
                amount: 1,
                is_lose_hp: false,
                nature: 0,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    );
    let state = resolve(tx.commit().state, owner, vec![1], 2);
    let state = resolve(state, owner, vec![other.raw()], 3);
    assert_eq!(state.players.get(other).unwrap().hp, other_hp + 1);
}

#[test]
fn zhiluo_source_and_npack_publish_other_end_draw() {
    let compiled = content_compiler::compile_source_named("miki_zhiluo", ZHILUO_SRC).unwrap();
    let trigger = compiled
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == MIKI_ZHILUO)
        .expect("zhiluo trigger");
    assert_eq!(trigger.event, "phase_end");
    assert_eq!(trigger.filter, "owner_alive_not_target");
    assert!(trigger.optional);
    let pack = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(pack.trigger_index.iter().any(|trigger| {
        trigger.skill_def_id == MIKI_ZHILUO && trigger.filter == "owner_alive_not_target"
    }));
}

#[test]
fn zhiluo_other_end_phase_draws_one() {
    let (state, owner, other) = setup(ZHILUO_SRC, "zhiluo-test", MIKI_ZHILUO, &[]);
    let before = state.hand_count(owner);
    let (state, requested) = fire_phase(state, other, TurnPhaseTag::End);
    assert!(requested);
    let state = resolve(state, owner, vec![1], 4);
    assert_eq!(state.hand_count(owner), before + 1);
}

#[test]
fn tuolao_source_and_npack_publish_end_swap() {
    let compiled = content_compiler::compile_source_named("hiroto_tuolao", TUOLAO_SRC).unwrap();
    let trigger = compiled
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == HIROTO_TUOLAO)
        .expect("tuolao trigger");
    assert_eq!(trigger.event, "phase_end");
    assert_eq!(trigger.filter, "owner_is_target");
    assert!(!trigger.optional);
    let pack = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(pack.trigger_index.iter().any(|trigger| {
        trigger.skill_def_id == HIROTO_TUOLAO && trigger.event == "phase_end"
    }));
}

#[test]
fn tuolao_end_phase_draws_three_and_swaps_huyu_for_zonglve_once() {
    let (state, owner, _) = setup(
        TUOLAO_SRC,
        "tuolao-test",
        HIROTO_TUOLAO,
        &[HIROTO_HUYU],
    );
    let before = state.hand_count(owner);
    assert!(has_def(&state, owner, HIROTO_HUYU));
    assert!(!has_def(&state, owner, HIROTO_ZONGLVE));
    let (state, requested) = fire_phase(state, owner, TurnPhaseTag::End);
    assert!(!requested);
    assert_eq!(state.hand_count(owner), before + 3);
    assert_eq!(once_mark(&state, owner, HIROTO_TUOLAO), 1);
    assert!(!has_def(&state, owner, HIROTO_HUYU));
    assert!(has_def(&state, owner, HIROTO_ZONGLVE));

    let after = state.hand_count(owner);
    let (state, _) = fire_phase(state, owner, TurnPhaseTag::End);
    assert_eq!(state.hand_count(owner), after);
    assert_eq!(once_mark(&state, owner, HIROTO_TUOLAO), 1);
}
