//! Golden coverage for KEY 得以 / 视心 / 梦终 rewritten from honest noops.

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

const YUZURU_DEYI: u32 = 3_890_409_763;
const YUZURU_KUNFEN: u32 = 2_231_368_330;
const YUZURU_QUJI: u32 = 2_805_568_225;
const YUZURU_WANGSHENG: u32 = 3_744_539_174;
const YUZURU_WUXIN: u32 = 3_940_257_990;
const AO_SHIXIN: u32 = 2_364_015_708;
const AO_DIEGUI: u32 = 2_867_140_701;
const AO_KUIHUN: u32 = 3_209_605_261;
const RIKI_MENGZHONG: u32 = 2_200_887_116;
const RIKI_WENJI: u32 = 2_666_301_691;
const RIKI_CHONGZHEN: u32 = 4_103_807_065;

const DEYI_SRC: &str = include_str!("../../../content/original/ported/key/yuzuru_deyi.skill");
const SHIXIN_SRC: &str = include_str!("../../../content/original/ported/key/ao_shixin.skill");
const MENGZHONG_SRC: &str =
    include_str!("../../../content/original/ported/key/riki_mengzhong.skill");
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/key/key.npack.json");

fn setup(source: &str, name: &str, skill_def: u32, extras: &[u32]) -> (GameState, PlayerId) {
    let compiled = content_compiler::compile_source_named(name, source).unwrap();
    let mut engine = Engine::new();
    engine.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(20_260_815_20),
        kind: CommandKind::StartGame {
            player_count: 2,
            rng_seed: 20_260_815_20,
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

fn kill_other(state: GameState, owner: PlayerId) -> GameState {
    let victim = state
        .players
        .alive_by_seat()
        .into_iter()
        .find(|player| *player != owner)
        .unwrap();
    let mut tx = Transaction::begin(state);
    run_effects(
        &mut tx,
        vec![EffectRequest::KillPlayer {
            player: victim,
            killer: owner,
        }],
    )
    .unwrap();
    tx.commit().state
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

fn assert_npack_trigger(skill: u32, event: &str, filter: &str) {
    let pack = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(pack.trigger_index.iter().any(|trigger| {
        trigger.skill_def_id == skill && trigger.event == event && trigger.filter == filter
    }));
}

#[test]
fn deyi_source_and_npack_publish_death_swap() {
    let compiled = content_compiler::compile_source_named("yuzuru_deyi", DEYI_SRC).unwrap();
    let trigger = compiled
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == YUZURU_DEYI)
        .expect("deyi trigger");
    assert_eq!(trigger.event, "death");
    assert_eq!(trigger.filter, "owner_alive_not_target");
    assert!(!trigger.optional);
    assert_npack_trigger(YUZURU_DEYI, "death", "owner_alive_not_target");
    let pack = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let actual = pack
        .modules
        .iter()
        .find(|module| module.name == compiled.modules[0].name)
        .expect("npack module for deyi");
    assert_eq!(actual.logic_hash, compiled.modules[0].logic_hash);
}

#[test]
fn deyi_on_other_death_swaps_to_wuxin_once() {
    let (mut state, owner) = setup(
        DEYI_SRC,
        "deyi-test",
        YUZURU_DEYI,
        &[YUZURU_KUNFEN, YUZURU_QUJI, YUZURU_WANGSHENG],
    );
    {
        let player = state.players.get_mut(owner).unwrap();
        player.max_hp = 4;
        player.hp = 2;
    }
    assert!(has_def(&state, owner, YUZURU_KUNFEN));
    assert!(has_def(&state, owner, YUZURU_QUJI));
    assert!(has_def(&state, owner, YUZURU_WANGSHENG));
    assert!(!has_def(&state, owner, YUZURU_WUXIN));

    let state = kill_other(state, owner);
    let player = state.players.get(owner).unwrap();
    assert_eq!(player.max_hp, 3);
    assert_eq!(player.hp, 3);
    assert_eq!(once_mark(&state, owner, YUZURU_DEYI), 1);
    assert!(!has_def(&state, owner, YUZURU_KUNFEN));
    assert!(!has_def(&state, owner, YUZURU_QUJI));
    assert!(!has_def(&state, owner, YUZURU_WANGSHENG));
    assert!(has_def(&state, owner, YUZURU_WUXIN));
}

#[test]
fn shixin_source_and_npack_publish_prepare_swap() {
    let compiled = content_compiler::compile_source_named("ao_shixin", SHIXIN_SRC).unwrap();
    let trigger = compiled
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == AO_SHIXIN)
        .expect("shixin trigger");
    assert_eq!(trigger.event, "phase_prepare");
    assert_eq!(trigger.filter, "owner_is_target");
    assert!(!trigger.optional);
    assert_npack_trigger(AO_SHIXIN, "phase_prepare", "owner_is_target");
}

#[test]
fn shixin_prepare_gains_max_hp_and_swaps_diegui_for_kuihun_once() {
    let (mut state, owner) = setup(SHIXIN_SRC, "shixin-test", AO_SHIXIN, &[AO_DIEGUI]);
    {
        let player = state.players.get_mut(owner).unwrap();
        player.max_hp = 3;
        player.hp = 3;
    }
    assert!(has_def(&state, owner, AO_DIEGUI));
    assert!(!has_def(&state, owner, AO_KUIHUN));

    let state = fire_prepare(state, owner);
    let player = state.players.get(owner).unwrap();
    assert_eq!(player.max_hp, 4);
    assert_eq!(player.hp, 4);
    assert_eq!(once_mark(&state, owner, AO_SHIXIN), 1);
    assert!(!has_def(&state, owner, AO_DIEGUI));
    assert!(has_def(&state, owner, AO_KUIHUN));

    let state = fire_prepare(state, owner);
    let player = state.players.get(owner).unwrap();
    assert_eq!(player.max_hp, 4);
    assert_eq!(player.hp, 4);
    assert_eq!(once_mark(&state, owner, AO_SHIXIN), 1);
}

#[test]
fn mengzhong_source_and_npack_publish_prepare_swap() {
    let compiled = content_compiler::compile_source_named("riki_mengzhong", MENGZHONG_SRC).unwrap();
    let trigger = compiled
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == RIKI_MENGZHONG)
        .expect("mengzhong trigger");
    assert_eq!(trigger.event, "phase_prepare");
    assert_eq!(trigger.filter, "owner_is_target");
    assert!(!trigger.optional);
    assert_npack_trigger(RIKI_MENGZHONG, "phase_prepare", "owner_is_target");
}

#[test]
fn mengzhong_prepare_swaps_wenji_for_chongzhen_once() {
    let (state, owner) = setup(
        MENGZHONG_SRC,
        "mengzhong-test",
        RIKI_MENGZHONG,
        &[RIKI_WENJI],
    );
    assert!(has_def(&state, owner, RIKI_WENJI));
    assert!(!has_def(&state, owner, RIKI_CHONGZHEN));

    let state = fire_prepare(state, owner);
    assert_eq!(once_mark(&state, owner, RIKI_MENGZHONG), 1);
    assert!(!has_def(&state, owner, RIKI_WENJI));
    assert!(has_def(&state, owner, RIKI_CHONGZHEN));

    let state = fire_prepare(state, owner);
    assert_eq!(once_mark(&state, owner, RIKI_MENGZHONG), 1);
}
