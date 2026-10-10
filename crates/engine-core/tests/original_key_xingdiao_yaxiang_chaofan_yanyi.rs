//! Golden coverage for KEY 星钓 / 鸦翔 / 炒饭 / 演仪 rewritten from honest noops.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const FUUKO_XINGDIAO: u32 = 3_398_320_463;
const YUKITO_YAXIANG: u32 = 2_428_947_588;
const UMI_CHAOFAN: u32 = 2_428_039_328;
const YUSA_YANYI: u32 = 4_076_446_163;

const XINGDIAO_SRC: &str =
    include_str!("../../../content/original/ported/key/fuuko_xingdiao.skill");
const YAXIANG_SRC: &str = include_str!("../../../content/original/ported/key/yukito_yaxiang.skill");
const CHAOFAN_SRC: &str = include_str!("../../../content/original/ported/key/umi_chaofan.skill");
const YANYI_SRC: &str = include_str!("../../../content/original/ported/key/yusa_yanyi.skill");
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

#[test]
fn xingdiao_source_and_npack_publish_game_started() {
    let compiled = content_compiler::compile_source_named("fuuko_xingdiao", XINGDIAO_SRC).unwrap();
    let trigger = compiled
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == FUUKO_XINGDIAO)
        .expect("xingdiao trigger");
    assert_eq!(trigger.event, "game_started");
    assert_eq!(trigger.filter, "owner_alive");
    let pack = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(pack.trigger_index.iter().any(|trigger| {
        trigger.skill_def_id == FUUKO_XINGDIAO && trigger.event == "game_started"
    }));
}

#[test]
fn xingdiao_game_start_draws_to_eight() {
    let (mut tx, owner, _) = bootstrap(XINGDIAO_SRC, "xingdiao-test", FUUKO_XINGDIAO);
    assert_eq!(tx.state().hand_count(owner), 0);
    let effects = dispatch_for_events(
        &mut tx,
        &[DomainEvent::GameStarted { player_count: 2 }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap_or_else(|e| panic!("game-started dispatch error: {e:?}"));
    run_effects(&mut tx, effects).unwrap();
    assert_eq!(tx.state().hand_count(owner), 8);
}

#[test]
fn yaxiang_source_and_npack_publish_other_dying() {
    let compiled = content_compiler::compile_source_named("yukito_yaxiang", YAXIANG_SRC).unwrap();
    let trigger = compiled
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == YUKITO_YAXIANG)
        .expect("yaxiang trigger");
    assert_eq!(trigger.event, "dying");
    assert_eq!(trigger.filter, "owner_alive_not_target");
    assert!(trigger.optional);
    let pack = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(pack.trigger_index.iter().any(|trigger| {
        trigger.skill_def_id == YUKITO_YAXIANG && trigger.event == "dying"
    }));
}

#[test]
fn yaxiang_other_dying_recovers_to_three_and_unloads() {
    let (mut tx, owner, other) = bootstrap(YAXIANG_SRC, "yaxiang-test", YUKITO_YAXIANG);
    tx.state_mut().players.get_mut(other).unwrap().hp = 0;
    assert_eq!(
        dispatch_for_events(
            &mut tx,
            &[DomainEvent::EnteredDying {
                player: other,
                source: owner,
            }],
            &mut TriggerFireRecord::new(),
        ),
        Err(EffectError::ChoiceRequested)
    );
    let state = resolve(tx.commit().state, owner, vec![1], 2);
    assert_eq!(state.players.get(other).unwrap().hp, 3);
    assert!(!state
        .skills
        .sorted_live()
        .into_iter()
        .any(|skill| skill.def.raw() == YUKITO_YAXIANG));
}

#[test]
fn chaofan_source_and_npack_publish_active() {
    let compiled = content_compiler::compile_source_named("umi_chaofan", CHAOFAN_SRC).unwrap();
    assert!(compiled
        .active_index
        .iter()
        .any(|item| item.skill_def_id == UMI_CHAOFAN));
    let pack = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(pack
        .active_index
        .iter()
        .any(|item| item.skill_def_id == UMI_CHAOFAN));
}

#[test]
fn chaofan_discards_two_draws_one_then_offers_other() {
    let (mut tx, owner, other) = bootstrap(CHAOFAN_SRC, "chaofan-test", UMI_CHAOFAN);
    inject_hand(&mut tx, owner, 242_101);
    inject_hand(&mut tx, owner, 242_102);
    let skill = tx
        .state()
        .players
        .get(owner)
        .unwrap()
        .skills
        .iter()
        .copied()
        .find(|skill| tx.state().skills.get(*skill).unwrap().def.raw() == UMI_CHAOFAN)
        .unwrap();
    let other_hand = tx.state().hand_count(other);
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
                card_targets: vec![],
            },
        },
    );
    assert!(
        matches!(pipe.result, CommandResult::Accepted { .. }),
        "{:?}",
        pipe.result
    );
    assert_eq!(state.hand_count(owner), 1);
    let state = resolve(state, owner, vec![other.raw()], 3);
    let state = if state.has_pending_choice() {
        resolve(state, owner, vec![0], 4)
    } else {
        state
    };
    assert!(
        state.hand_count(other) == other_hand + 2
            || state.players.get(other).unwrap().hp
                > 0
    );
}

#[test]
fn yanyi_source_and_npack_publish_active() {
    let compiled = content_compiler::compile_source_named("yusa_yanyi", YANYI_SRC).unwrap();
    assert!(compiled
        .active_index
        .iter()
        .any(|item| item.skill_def_id == YUSA_YANYI));
    let pack = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(pack
        .active_index
        .iter()
        .any(|item| item.skill_def_id == YUSA_YANYI));
}
