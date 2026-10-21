//! Golden coverage for KEY 不悔 / 列阵 / 守琴.

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

const KUD_BUHUI: u32 = 4_166_321_790;
const KANO_LIEZHEN: u32 = 3_815_378_594;
const NAO_SHOUQING2: u32 = 4_259_091_944;
const BUHUI_SRC: &str = include_str!("../../../../content/original/ported/key/kud_buhui.skill");
const LIEZHEN_SRC: &str =
    include_str!("../../../../content/original/ported/key/kano_liezhen.skill");
const SHOUQING_SRC: &str =
    include_str!("../../../../content/original/ported/key/nao_shouqing2.skill");
const PACKAGE: &[u8] = include_bytes!("../../../../content/original/ported/key/key.npack.json");

fn bootstrap(source: &str, name: &str, skill_def: u32) -> (Transaction, PlayerId) {
    let compiled = content_compiler::compile_source_named(name, source).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 2, u64::from(skill_def), "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content)
        .load_package(&compiled)
        .unwrap();
    tx.state_mut().turn.phase = Phase::Play;
    let owner = tx.state().turn.current;
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::DiscardFromHand {
        player: owner,
        count: u32::MAX,
    });
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(skill_def),
    });
    run_effects(&mut tx, effects).unwrap();
    (tx, owner)
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
fn buhui_source_and_npack_publish_dying_recover() {
    let compiled = content_compiler::compile_source_named("kud_buhui", BUHUI_SRC).unwrap();
    assert!(compiled.trigger_index.iter().any(|trigger| {
        trigger.skill_def_id == KUD_BUHUI && trigger.event == "dying" && trigger.optional
    }));
    let pack = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(pack
        .trigger_index
        .iter()
        .any(|trigger| trigger.skill_def_id == KUD_BUHUI && trigger.event == "dying"));
}

#[test]
fn buhui_recovers_to_two_when_dying() {
    let (mut tx, owner) = bootstrap(BUHUI_SRC, "buhui-test", KUD_BUHUI);
    tx.state_mut().players.get_mut(owner).unwrap().hp = 0;
    assert_eq!(
        dispatch_for_events(
            &mut tx,
            &[DomainEvent::EnteredDying {
                player: owner,
                source: PlayerId::NONE,
            }],
            &mut TriggerFireRecord::new(),
        ),
        Err(EffectError::ChoiceRequested)
    );
    let state = resolve(tx.commit().state, owner, vec![1], 2);
    assert_eq!(state.players.get(owner).unwrap().hp, 2);
}

#[test]
fn liezhen_source_and_npack_publish_end_draw() {
    let compiled = content_compiler::compile_source_named("kano_liezhen", LIEZHEN_SRC).unwrap();
    assert!(compiled.trigger_index.iter().any(|trigger| {
        trigger.skill_def_id == KANO_LIEZHEN && trigger.event == "phase_end" && trigger.optional
    }));
    let pack = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(pack
        .trigger_index
        .iter()
        .any(|trigger| trigger.skill_def_id == KANO_LIEZHEN && trigger.event == "phase_end"));
}

#[test]
fn liezhen_end_phase_draws_two() {
    let (tx, owner) = bootstrap(LIEZHEN_SRC, "liezhen-test", KANO_LIEZHEN);
    let before = tx.state().hand_count(owner);
    let mut tx = tx;
    assert_eq!(
        dispatch_for_events(
            &mut tx,
            &[DomainEvent::TurnStateChanged {
                current: owner,
                phase: TurnPhaseTag::End,
            }],
            &mut TriggerFireRecord::new(),
        ),
        Err(EffectError::ChoiceRequested)
    );
    let state = resolve(tx.commit().state, owner, vec![1], 3);
    assert_eq!(state.hand_count(owner), before + 2);
}

#[test]
fn shouqing_source_and_npack_publish_tao() {
    let compiled = content_compiler::compile_source_named("nao_shouqing2", SHOUQING_SRC).unwrap();
    assert!(compiled
        .view_as_index
        .iter()
        .any(|view| view.skill_def_id == NAO_SHOUQING2 && view.as_kind == "tao"));
    let pack = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(pack
        .view_as_index
        .iter()
        .any(|view| view.skill_def_id == NAO_SHOUQING2 && view.as_kind == "tao"));
}

#[test]
fn shouqing_converts_hand_card_to_tao_when_wounded() {
    let (mut tx, owner) = bootstrap(SHOUQING_SRC, "shouqing-test", NAO_SHOUQING2);
    tx.state_mut().players.get_mut(owner).unwrap().hp = 1;
    let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(193_301),
        kind: CardKind::Shan,
        suit: Suit::Club,
        rank: 2,
        weapon_range: 0,
        zone: hand,
        owner,
        face_up: true,
    });
    tx.state_mut().zones.get_mut(hand).unwrap().cards.push(card);
    let skill = tx
        .state()
        .players
        .get(owner)
        .unwrap()
        .skills
        .iter()
        .copied()
        .find(|skill| tx.state().skills.get(*skill).unwrap().def.raw() == NAO_SHOUQING2)
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
                targets: vec![owner],
                card_targets: vec![card],
            },
        },
    );
    assert!(
        matches!(pipe.result, CommandResult::Accepted { .. }),
        "{:?}",
        pipe.result
    );
    assert_eq!(state.players.get(owner).unwrap().hp, 2);
}
