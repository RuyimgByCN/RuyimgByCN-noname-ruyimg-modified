//! Golden coverage for KEY 替身 / 潜梦 / 劝请 rewritten from honest noops.

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

const MIDORI_TISHEN: u32 = 2_348_148_371;
const MIO_TISHEN: u32 = 4_056_753_334;
const MIA_QIANMENG: u32 = 2_319_632_885;
const AKANE_QUANQING: u32 = 2_806_428_173;

const MIDORI_SRC: &str =
    include_str!("../../../../content/original/ported/key/midori_tishen.skill");
const MIO_SRC: &str = include_str!("../../../../content/original/ported/key/mio_tishen.skill");
const QIANMENG_SRC: &str =
    include_str!("../../../../content/original/ported/key/mia_qianmeng.skill");
const QUANQING_SRC: &str =
    include_str!("../../../../content/original/ported/key/akane_quanqing.skill");
const PACKAGE: &[u8] = include_bytes!("../../../../content/original/ported/key/key.npack.json");

fn bootstrap(source: &str, name: &str, skill_def: u32) -> (Transaction, PlayerId, PlayerId) {
    let compiled = content_compiler::compile_source_named(name, source).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 2, u64::from(skill_def), "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content)
        .load_package(&compiled)
        .unwrap();
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

fn assert_tishen_source_and_npack(source: &str, name: &str, skill_def: u32) {
    let compiled = content_compiler::compile_source_named(name, source).unwrap();
    let trigger = compiled
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == skill_def)
        .expect("tishen trigger");
    assert_eq!(trigger.event, "phase_prepare");
    assert_eq!(trigger.filter, "owner_is_target_wounded");
    assert!(trigger.optional);
    let pack = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(pack
        .trigger_index
        .iter()
        .any(|trigger| { trigger.skill_def_id == skill_def && trigger.event == "phase_prepare" }));
}

fn tishen_prepare_recovers_and_draws(source: &str, name: &str, skill_def: u32) {
    let (mut tx, owner, _) = bootstrap(source, name, skill_def);
    tx.state_mut().players.get_mut(owner).unwrap().hp = 2;
    let max_hp = tx.state().players.get(owner).unwrap().max_hp as i32;
    let lost = (max_hp - 2) as u32;
    let hand_before = tx.state().hand_count(owner);
    assert_eq!(
        dispatch_for_events(
            &mut tx,
            &[DomainEvent::TurnStateChanged {
                current: owner,
                phase: TurnPhaseTag::Prepare,
            }],
            &mut TriggerFireRecord::new(),
        ),
        Err(EffectError::ChoiceRequested)
    );
    let state = resolve(tx.commit().state, owner, vec![1], 2);
    assert_eq!(state.players.get(owner).unwrap().hp, max_hp);
    assert_eq!(state.hand_count(owner), hand_before + lost);
    assert!(!state
        .skills
        .sorted_live()
        .into_iter()
        .any(|skill| skill.def.raw() == skill_def));
}

#[test]
fn midori_tishen_source_and_npack_publish_prepare() {
    assert_tishen_source_and_npack(MIDORI_SRC, "midori_tishen", MIDORI_TISHEN);
}

#[test]
fn midori_tishen_prepare_recovers_and_draws() {
    tishen_prepare_recovers_and_draws(MIDORI_SRC, "midori-tishen-test", MIDORI_TISHEN);
}

#[test]
fn mio_tishen_source_and_npack_publish_prepare() {
    assert_tishen_source_and_npack(MIO_SRC, "mio_tishen", MIO_TISHEN);
}

#[test]
fn mio_tishen_prepare_recovers_and_draws() {
    tishen_prepare_recovers_and_draws(MIO_SRC, "mio-tishen-test", MIO_TISHEN);
}

#[test]
fn qianmeng_source_and_npack_publish_game_started() {
    let compiled = content_compiler::compile_source_named("mia_qianmeng", QIANMENG_SRC).unwrap();
    let trigger = compiled
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == MIA_QIANMENG)
        .expect("qianmeng trigger");
    assert_eq!(trigger.event, "game_started");
    assert_eq!(trigger.filter, "owner_alive");
    let pack = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(pack.trigger_index.iter().any(|trigger| {
        trigger.skill_def_id == MIA_QIANMENG && trigger.event == "game_started"
    }));
}

#[test]
fn qianmeng_game_start_draws_one() {
    let (mut tx, owner, _) = bootstrap(QIANMENG_SRC, "qianmeng-test", MIA_QIANMENG);
    let before = tx.state().hand_count(owner);
    let effects = dispatch_for_events(
        &mut tx,
        &[DomainEvent::GameStarted { player_count: 2 }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap_or_else(|e| panic!("game-started dispatch error: {e:?}"));
    run_effects(&mut tx, effects).unwrap();
    assert_eq!(tx.state().hand_count(owner), before + 1);
}

#[test]
fn quanqing_source_and_npack_publish_active() {
    let compiled = content_compiler::compile_source_named("akane_quanqing", QUANQING_SRC).unwrap();
    assert!(compiled
        .active_index
        .iter()
        .any(|item| item.skill_def_id == AKANE_QUANQING));
    let pack = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(pack
        .active_index
        .iter()
        .any(|item| item.skill_def_id == AKANE_QUANQING));
}

#[test]
fn quanqing_discards_one_and_lets_other_draw() {
    let (mut tx, owner, other) = bootstrap(QUANQING_SRC, "quanqing-test", AKANE_QUANQING);
    let card = inject_hand(&mut tx, owner, 280_101);
    let skill = tx
        .state()
        .players
        .get(owner)
        .unwrap()
        .skills
        .iter()
        .copied()
        .find(|skill| tx.state().skills.get(*skill).unwrap().def.raw() == AKANE_QUANQING)
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
                targets: vec![other],
                card_targets: vec![card],
            },
        },
    );
    assert!(
        matches!(pipe.result, CommandResult::Accepted { .. }),
        "{:?}",
        pipe.result
    );
    assert_eq!(state.hand_count(owner), 0);
    assert_eq!(state.hand_count(other), other_hand + 1);
}
