//! Golden coverage for KEY 窥魂 / 坚忍 / 拾慧 / 精策 rewritten from honest noops.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, Suit, TurnPhaseTag,
};
use engine_core::command::apply_command;
use engine_core::effect::{run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use engine_core::Engine;
use rule_api::EffectRequest;

const AO_KUIHUN: u32 = 3_209_605_261;
const SUNOHARA_JIANREN: u32 = 3_420_828_175;
const MIA_SHIHUI: u32 = 3_357_314_746;
const KYOKO_JINGCE: u32 = 2_335_248_353;

const KUIHUN_SRC: &str = include_str!("../../../content/original/ported/key/ao_kuihun.skill");
const JIANREN_SRC: &str =
    include_str!("../../../content/original/ported/key/sunohara_jianren.skill");
const SHIHUI_SRC: &str = include_str!("../../../content/original/ported/key/mia_shihui.skill");
const JINGCE_SRC: &str = include_str!("../../../content/original/ported/key/kyoko_jingce.skill");
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/key/key.npack.json");

fn setup(source: &str, name: &str, skill_def: u32) -> (GameState, PlayerId, PlayerId) {
    let compiled = content_compiler::compile_source_named(name, source).unwrap();
    let mut engine = Engine::new();
    engine.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(20_260_815_40),
        kind: CommandKind::StartGame {
            player_count: 2,
            rng_seed: 20_260_815_40,
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

fn inject_hand(state: &mut GameState, owner: PlayerId) -> CardId {
    let hand = state.zones.find(ZoneKind::Hand, owner).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(335_701),
        kind: CardKind::Sha,
        suit: Suit::Spade,
        rank: 7,
        weapon_range: 0,
        zone: hand,
        owner,
        face_up: true,
    });
    state.zones.get_mut(hand).unwrap().cards.push(card);
    card
}

#[test]
fn kuihun_source_and_npack_publish_other_dying_draw() {
    let compiled = content_compiler::compile_source_named("ao_kuihun", KUIHUN_SRC).unwrap();
    let trigger = compiled
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == AO_KUIHUN)
        .expect("kuihun trigger");
    assert_eq!(trigger.event, "dying");
    assert_eq!(trigger.filter, "owner_alive_not_target");
    assert!(!trigger.optional);
    let pack = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(pack.trigger_index.iter().any(|trigger| {
        trigger.skill_def_id == AO_KUIHUN && trigger.filter == "owner_alive_not_target"
    }));
}

#[test]
fn kuihun_draws_one_when_other_enters_dying() {
    let (state, owner, other) = setup(KUIHUN_SRC, "kuihun-test", AO_KUIHUN);
    let before = state.hand_count(owner);
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
    assert_eq!(tx.state().hand_count(owner), before + 1);
}

#[test]
fn jianren_source_and_npack_publish_damage_draw() {
    let compiled = content_compiler::compile_source_named("sunohara_jianren", JIANREN_SRC).unwrap();
    let trigger = compiled
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == SUNOHARA_JIANREN)
        .expect("jianren trigger");
    assert_eq!(trigger.event, "damage");
    assert_eq!(trigger.filter, "owner_is_target");
    assert!(trigger.optional);
    let pack = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(pack.trigger_index.iter().any(|trigger| {
        trigger.skill_def_id == SUNOHARA_JIANREN && trigger.event == "damage"
    }));
}

#[test]
fn jianren_after_damage_lets_chosen_player_draw_one() {
    let (state, owner, other) = setup(JIANREN_SRC, "jianren-test", SUNOHARA_JIANREN);
    let other_hand = state.hand_count(other);
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
    assert_eq!(state.hand_count(other), other_hand + 1);
}

#[test]
fn shihui_source_and_npack_publish_end_discard_recover() {
    let compiled = content_compiler::compile_source_named("mia_shihui", SHIHUI_SRC).unwrap();
    let trigger = compiled
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == MIA_SHIHUI)
        .expect("shihui trigger");
    assert_eq!(trigger.event, "phase_end");
    assert_eq!(trigger.filter, "owner_is_target");
    assert!(!trigger.optional);
    let pack = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(pack
        .trigger_index
        .iter()
        .any(|trigger| trigger.skill_def_id == MIA_SHIHUI && trigger.event == "phase_end"));
}

#[test]
fn shihui_end_phase_recovers_then_discards_one() {
    let (mut state, owner, _) = setup(SHIHUI_SRC, "shihui-test", MIA_SHIHUI);
    let card = inject_hand(&mut state, owner);
    {
        let player = state.players.get_mut(owner).unwrap();
        player.hp = player.max_hp as i32 - 1;
    }
    let before_hand = state.hand_count(owner);
    let before_hp = state.players.get(owner).unwrap().hp;
    let mut tx = Transaction::begin(state);
    let effects = dispatch_for_events(
        &mut tx,
        &[DomainEvent::TurnStateChanged {
            current: owner,
            phase: TurnPhaseTag::End,
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap_or_else(|e| panic!("shihui dispatch error: {e:?}"));
    assert_eq!(run_effects(&mut tx, effects), Err(EffectError::ChoiceRequested));
    assert_eq!(tx.state().players.get(owner).unwrap().hp, before_hp + 1);
    let state = resolve(tx.commit().state, owner, vec![card.raw()], 5);
    assert_eq!(state.hand_count(owner), before_hand - 1);
    assert_eq!(state.players.get(owner).unwrap().hp, before_hp + 1);
}

#[test]
fn jingce_source_and_npack_publish_end_draw() {
    let compiled = content_compiler::compile_source_named("kyoko_jingce", JINGCE_SRC).unwrap();
    let trigger = compiled
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == KYOKO_JINGCE)
        .expect("jingce trigger");
    assert_eq!(trigger.event, "phase_end");
    assert_eq!(trigger.filter, "owner_is_target");
    assert!(trigger.optional);
    let pack = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(pack
        .trigger_index
        .iter()
        .any(|trigger| trigger.skill_def_id == KYOKO_JINGCE && trigger.event == "phase_end"));
}

#[test]
fn jingce_end_phase_draws_two() {
    let (state, owner, _) = setup(JINGCE_SRC, "jingce-test", KYOKO_JINGCE);
    let before = state.hand_count(owner);
    let (state, requested) = fire_phase(state, owner, TurnPhaseTag::End);
    assert!(requested);
    let state = resolve(state, owner, vec![1], 4);
    assert_eq!(state.hand_count(owner), before + 2);
}
