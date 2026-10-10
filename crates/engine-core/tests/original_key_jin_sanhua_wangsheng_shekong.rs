//! Golden coverage for KEY 晋势 / 散华 / 往生 / 设恐 rewritten from honest noops.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, Suit, TurnPhaseTag,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const KOTORI_SKILL_JIN: u32 = 3_135_780_791;
const SHIZUKU_SANHUA: u32 = 3_952_218_459;
const YUZURU_WANGSHENG: u32 = 3_744_539_174;
const NK_SHEKONG: u32 = 3_537_526_857;

const JIN_SRC: &str = include_str!("../../../content/original/ported/key/kotori_skill_jin.skill");
const SANHUA_SRC: &str = include_str!("../../../content/original/ported/key/shizuku_sanhua.skill");
const WANGSHENG_SRC: &str =
    include_str!("../../../content/original/ported/key/yuzuru_wangsheng.skill");
const SHEKONG_SRC: &str = include_str!("../../../content/original/ported/key/nk_shekong.skill");
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/key/key.npack.json");

fn bootstrap(source: &str, name: &str, skill_def: u32) -> (Transaction, PlayerId, PlayerId) {
    let compiled = content_compiler::compile_source_named(name, source).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 2, u64::from(skill_def), "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&compiled).unwrap();
    tx.state_mut().turn.phase = Phase::Play;
    let owner = tx.state().turn.current;
    let other = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .find(|player| *player != owner)
        .unwrap();
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
        skill_def: SkillDefId::new(skill_def),
    });
    run_effects(&mut tx, effects).unwrap();
    (tx, owner, other)
}

fn inject_hand(tx: &mut Transaction, owner: PlayerId, def: u32) -> CardId {
    let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind: CardKind::Shan,
        suit: Suit::Club,
        rank: 6,
        weapon_range: 0,
        zone: hand,
        owner,
        face_up: true,
    });
    tx.state_mut().zones.get_mut(hand).unwrap().cards.push(card);
    card
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

fn once_mark(state: &GameState, owner: PlayerId, skill: u32) -> i32 {
    state
        .players
        .get(owner)
        .and_then(|player| player.marks.iter().find(|(id, _)| *id == skill))
        .map(|(_, count)| *count)
        .unwrap_or(0)
}

#[test]
fn jin_source_and_npack_publish_draw_end() {
    let compiled = content_compiler::compile_source_named("kotori_skill_jin", JIN_SRC).unwrap();
    let trigger = compiled
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == KOTORI_SKILL_JIN)
        .expect("jin trigger");
    assert_eq!(trigger.event, "phase_draw_end");
    assert_eq!(trigger.filter, "owner_is_target");
    assert!(trigger.optional);
    let pack = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(pack.trigger_index.iter().any(|trigger| {
        trigger.skill_def_id == KOTORI_SKILL_JIN && trigger.event == "phase_draw_end"
    }));
}

#[test]
fn jin_draw_phase_end_draws_one() {
    let (tx, owner, _) = bootstrap(JIN_SRC, "jin-test", KOTORI_SKILL_JIN);
    let before = tx.state().hand_count(owner);
    let mut tx = tx;
    assert_eq!(
        dispatch_for_events(
            &mut tx,
            &[DomainEvent::PhaseEnded {
                current: owner,
                phase: TurnPhaseTag::Draw,
                damage_sources: 0,
                damaged_players: 0,
                card_targeted_other: false,
                current_dealt_damage_other: false,
                phase_targeted_players: 0,
                phase_hp_changed_players: 0,
            }],
            &mut TriggerFireRecord::new(),
        ),
        Err(EffectError::ChoiceRequested)
    );
    let state = resolve(tx.commit().state, owner, vec![1], 2);
    assert_eq!(state.hand_count(owner), before + 1);
}

#[test]
fn sanhua_source_and_npack_publish_death_draw() {
    let compiled = content_compiler::compile_source_named("shizuku_sanhua", SANHUA_SRC).unwrap();
    let trigger = compiled
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == SHIZUKU_SANHUA)
        .expect("sanhua trigger");
    assert_eq!(trigger.event, "dying");
    assert_eq!(trigger.filter, "owner_is_target");
    let pack = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(pack
        .trigger_index
        .iter()
        .any(|trigger| trigger.skill_def_id == SHIZUKU_SANHUA && trigger.event == "dying"));
}

#[test]
fn sanhua_death_lets_chosen_other_draw_four() {
    let (tx, owner, other) = bootstrap(SANHUA_SRC, "sanhua-test", SHIZUKU_SANHUA);
    let other_hand = tx.state().hand_count(other);
    let mut tx = tx;
    let effects = dispatch_for_events(
        &mut tx,
        &[DomainEvent::EnteredDying {
            player: owner,
            source: other,
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap_or_else(|e| panic!("dying dispatch error: {e:?}"));
    assert_eq!(run_effects(&mut tx, effects), Err(EffectError::ChoiceRequested));
    let state = resolve(tx.commit().state, owner, vec![other.raw()], 3);
    assert_eq!(state.hand_count(other), other_hand + 4);
}

#[test]
fn wangsheng_source_and_npack_publish_dying_recover() {
    let compiled =
        content_compiler::compile_source_named("yuzuru_wangsheng", WANGSHENG_SRC).unwrap();
    let trigger = compiled
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == YUZURU_WANGSHENG)
        .expect("wangsheng trigger");
    assert_eq!(trigger.event, "dying");
    assert_eq!(trigger.filter, "owner_is_target");
    assert!(!trigger.optional);
    let pack = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(pack.trigger_index.iter().any(|trigger| {
        trigger.skill_def_id == YUZURU_WANGSHENG && trigger.event == "dying"
    }));
}

#[test]
fn wangsheng_dying_recovers_to_one_once() {
    let (mut tx, owner, other) = bootstrap(WANGSHENG_SRC, "wangsheng-test", YUZURU_WANGSHENG);
    tx.state_mut().players.get_mut(owner).unwrap().hp = 0;
    let effects = dispatch_for_events(
        &mut tx,
        &[DomainEvent::EnteredDying {
            player: owner,
            source: other,
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap_or_else(|e| panic!("dying dispatch error: {e:?}"));
    run_effects(&mut tx, effects).unwrap();
    assert_eq!(tx.state().players.get(owner).unwrap().hp, 1);
    assert_eq!(once_mark(tx.state(), owner, YUZURU_WANGSHENG), 1);

    tx.state_mut().players.get_mut(owner).unwrap().hp = 0;
    let effects = dispatch_for_events(
        &mut tx,
        &[DomainEvent::EnteredDying {
            player: owner,
            source: other,
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap_or_else(|e| panic!("second dying dispatch error: {e:?}"));
    run_effects(&mut tx, effects).unwrap();
    assert_eq!(tx.state().players.get(owner).unwrap().hp, 0);
}

#[test]
fn shekong_source_and_npack_publish_active() {
    let compiled = content_compiler::compile_source_named("nk_shekong", SHEKONG_SRC).unwrap();
    assert!(compiled
        .active_index
        .iter()
        .any(|item| item.skill_def_id == NK_SHEKONG));
    let pack = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(pack
        .active_index
        .iter()
        .any(|item| item.skill_def_id == NK_SHEKONG));
}

#[test]
fn shekong_discards_redraws_and_lets_other_draw_one() {
    let (mut tx, owner, other) = bootstrap(SHEKONG_SRC, "shekong-test", NK_SHEKONG);
    let card = inject_hand(&mut tx, owner, 180_201);
    let owner_hand = tx.state().hand_count(owner);
    let other_hand = tx.state().hand_count(other);
    let skill = tx
        .state()
        .players
        .get(owner)
        .unwrap()
        .skills
        .iter()
        .copied()
        .find(|skill| tx.state().skills.get(*skill).unwrap().def.raw() == NK_SHEKONG)
        .unwrap();
    let state = tx.commit().state;
    let (state, pipe) = apply_command(
        state.clone(),
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(state.revision),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::ActivateSkill {
                skill,
                targets: vec![],
                card_targets: vec![card],
            },
        },
    );
    assert!(
        matches!(pipe.result, CommandResult::Accepted { .. }),
        "{:?}",
        pipe.result
    );
    let state = resolve(state, owner, vec![other.raw()], 4);
    assert_eq!(state.hand_count(owner), owner_hand);
    assert_eq!(state.hand_count(other), other_hand + 1);
}
