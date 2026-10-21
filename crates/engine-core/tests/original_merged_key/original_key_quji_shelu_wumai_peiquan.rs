//! Golden coverage for KEY 曲继 / 射鹿 / 五脉 / 配券 rewritten from honest noops.

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

const YUZURU_QUJI: u32 = 2_805_568_225;
const ASARA_SHELU: u32 = 4_161_128_201;
const LIYINGXIA_WUMAI: u32 = 2_506_272_675;
const MINAGI_PEIQUAN: u32 = 3_941_246_023;

const QUJI_SRC: &str = include_str!("../../../../content/original/ported/key/yuzuru_quji.skill");
const SHELU_SRC: &str = include_str!("../../../../content/original/ported/key/asara_shelu.skill");
const WUMAI_SRC: &str =
    include_str!("../../../../content/original/ported/key/liyingxia_wumai.skill");
const PEIQUAN_SRC: &str =
    include_str!("../../../../content/original/ported/key/minagi_peiquan.skill");
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

fn skill_id(tx: &Transaction, owner: PlayerId, skill_def: u32) -> engine_api::SkillId {
    tx.state()
        .players
        .get(owner)
        .unwrap()
        .skills
        .iter()
        .copied()
        .find(|skill| tx.state().skills.get(*skill).unwrap().def.raw() == skill_def)
        .unwrap()
}

fn activate(
    state: GameState,
    actor: PlayerId,
    skill: engine_api::SkillId,
    targets: Vec<PlayerId>,
    card_targets: Vec<CardId>,
    key: u64,
) -> GameState {
    let (state, pipe) = apply_command(
        state.clone(),
        Command {
            actor,
            expected_revision: ExpectedRevision(state.revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ActivateSkill {
                skill,
                targets,
                card_targets,
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
fn quji_source_and_npack_publish_active() {
    let compiled = content_compiler::compile_source_named("yuzuru_quji", QUJI_SRC).unwrap();
    assert!(compiled
        .active_index
        .iter()
        .any(|item| item.skill_def_id == YUZURU_QUJI));
    let pack = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(pack
        .active_index
        .iter()
        .any(|item| item.skill_def_id == YUZURU_QUJI));
}

#[test]
fn quji_discards_one_and_recovers_wounded_other() {
    let (mut tx, owner, other) = bootstrap(QUJI_SRC, "quji-test", YUZURU_QUJI);
    let card = inject_hand(&mut tx, owner, 280_201);
    tx.state_mut().players.get_mut(other).unwrap().hp = 2;
    let skill = skill_id(&tx, owner, YUZURU_QUJI);
    let state = activate(tx.commit().state, owner, skill, vec![other], vec![card], 1);
    assert_eq!(state.hand_count(owner), 0);
    assert_eq!(state.players.get(other).unwrap().hp, 3);
}

#[test]
fn shelu_source_and_npack_publish_active() {
    let compiled = content_compiler::compile_source_named("asara_shelu", SHELU_SRC).unwrap();
    assert!(compiled
        .active_index
        .iter()
        .any(|item| item.skill_def_id == ASARA_SHELU));
    let pack = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(pack
        .active_index
        .iter()
        .any(|item| item.skill_def_id == ASARA_SHELU));
}

#[test]
fn shelu_discards_one_and_draws_two() {
    let (mut tx, owner, _) = bootstrap(SHELU_SRC, "shelu-test", ASARA_SHELU);
    let card = inject_hand(&mut tx, owner, 416_101);
    let skill = skill_id(&tx, owner, ASARA_SHELU);
    let state = activate(tx.commit().state, owner, skill, vec![], vec![card], 1);
    assert_eq!(state.hand_count(owner), 2);
}

#[test]
fn wumai_source_and_npack_publish_turn_begin() {
    let compiled = content_compiler::compile_source_named("liyingxia_wumai", WUMAI_SRC).unwrap();
    let trigger = compiled
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == LIYINGXIA_WUMAI)
        .expect("wumai trigger");
    assert_eq!(trigger.event, "phase_begin");
    assert_eq!(trigger.filter, "owner_is_target");
    assert!(trigger.optional);
    let pack = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(pack.trigger_index.iter().any(|trigger| {
        trigger.skill_def_id == LIYINGXIA_WUMAI && trigger.event == "phase_begin"
    }));
}

#[test]
fn wumai_turn_begin_draws_one() {
    let (tx, owner, _) = bootstrap(WUMAI_SRC, "wumai-test", LIYINGXIA_WUMAI);
    let before = tx.state().hand_count(owner);
    let mut tx = tx;
    assert_eq!(
        dispatch_for_events(
            &mut tx,
            &[DomainEvent::TurnBegan { player: owner }],
            &mut TriggerFireRecord::new(),
        ),
        Err(EffectError::ChoiceRequested)
    );
    let state = resolve(tx.commit().state, owner, vec![1], 2);
    assert_eq!(state.hand_count(owner), before + 1);
}

#[test]
fn peiquan_source_and_npack_publish_active() {
    let compiled = content_compiler::compile_source_named("minagi_peiquan", PEIQUAN_SRC).unwrap();
    assert!(compiled
        .active_index
        .iter()
        .any(|item| item.skill_def_id == MINAGI_PEIQUAN));
    let pack = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(pack
        .active_index
        .iter()
        .any(|item| item.skill_def_id == MINAGI_PEIQUAN));
}

#[test]
fn peiquan_can_draw_two_instead_of_damage() {
    let (tx, owner, other) = bootstrap(PEIQUAN_SRC, "peiquan-test", MINAGI_PEIQUAN);
    let skill = skill_id(&tx, owner, MINAGI_PEIQUAN);
    let before = tx.state().hand_count(owner);
    let other_hp = tx.state().players.get(other).unwrap().hp;
    let state = activate(tx.commit().state, owner, skill, vec![other], vec![], 1);
    let state = resolve(state, owner, vec![2], 2);
    assert_eq!(state.hand_count(owner), before + 2);
    assert_eq!(state.players.get(other).unwrap().hp, other_hp);
}
