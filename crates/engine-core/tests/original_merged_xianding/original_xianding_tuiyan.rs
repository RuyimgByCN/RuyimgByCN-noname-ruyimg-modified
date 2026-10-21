//! 第一百八十四批：xianding「推演」（管辂 tuiyan）观星原语外溢，零新原语。
//! 上游：出牌阶段开始时（frequent），观看牌堆顶三张牌，以任意顺序放回牌堆顶
//! （viewCards + reverse insert）。
//! 实现：phase_play + owner_is_target（forced）→ guanxing_top(self, 3)。
//! 近似（登记债）：guanxing_top 允许逐张顶/底重排（pxlangya 同债）——能力超集，
//! 信息面等价。

use engine_api::{
    ChoiceId, Command, CommandKind, CommandResult, DomainEvent, ExpectedRevision, IdempotencyKey,
    PlayerId, SkillDefId, TurnPhaseTag,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{GameState, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const TUIYAN: u32 = 2_256_277_010;
const GUANLU: u32 = 4_222_425_710;
const SOURCE: &str = include_str!("../../../../content/original/ported/xianding/tuiyan.skill");
const XIANDING: &[u8] =
    include_bytes!("../../../../content/original/ported/xianding/xianding.npack.json");

fn setup(seed: u64) -> (GameState, PlayerId) {
    let compiled = content_compiler::compile_source_named("tuiyan-test", SOURCE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 2, seed, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content)
        .load_package(&compiled)
        .unwrap();
    let players = tx.state().players.alive_by_seat();
    let owner = players[0];
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(TUIYAN),
    });
    run_effects(&mut tx, effects).unwrap();
    (tx.commit().state, owner)
}

fn fire_play(state: GameState, owner: PlayerId) -> (GameState, Result<(), EffectError>) {
    let mut tx = Transaction::begin(state);
    let effects = dispatch_for_events(
        &mut tx,
        &[DomainEvent::TurnStateChanged {
            current: owner,
            phase: TurnPhaseTag::Play,
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    let outcome = run_effects(&mut tx, effects);
    (tx.commit().state, outcome.map(|_| ()))
}

fn resolve(state: GameState, owner: PlayerId, pick: u32, key: u64) -> GameState {
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    let revision = state.revision;
    let (next, result) = apply_command(
        state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections: vec![pick],
                cancelled: false,
            },
        },
    );
    assert!(
        matches!(result.result, CommandResult::Accepted { .. }),
        "guanxing round rejected: {:?}",
        result.result
    );
    next
}

#[test]
fn package_publishes_tuiyan_contract() {
    let package = content_schema::ContentPackage::from_json_bytes(XIANDING).unwrap();
    assert!(package.manifest.rule_vm_abi_version >= 681);
    let trigger = package
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == TUIYAN)
        .unwrap();
    assert_eq!(trigger.event, "phase_play");
    assert_eq!(trigger.filter, "owner_is_target");
    assert!(!trigger.optional);
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == GUANLU)
        .expect("guanlu in roster");
    assert!(character.skills.contains(&TUIYAN));
}

#[test]
fn play_phase_start_opens_three_card_view_loop() {
    let (state, owner) = setup(20_260_909_07);
    let (state, outcome) = fire_play(state, owner);
    assert!(
        matches!(outcome, Err(EffectError::ChoiceRequested)),
        "view loop must open, got {outcome:?}"
    );
    let mut next = state;
    let mut key = 20_260_909_17;
    for round in 0..3 {
        assert!(next.has_pending_choice(), "round {round} must stay open");
        next = resolve(next, owner, 1, key);
        key += 1;
    }
    assert!(!next.has_pending_choice(), "loop must close after 3 rounds");
    let processing = next
        .zones
        .find(ZoneKind::Processing, PlayerId::NONE)
        .unwrap();
    assert!(
        next.zones.get(processing).unwrap().cards.is_empty(),
        "processing must drain after the view settles"
    );
}

#[test]
fn other_player_phase_does_not_trigger() {
    let (state, owner) = setup(20_260_909_08);
    let mut tx = Transaction::begin(state);
    let others = tx.state().players.alive_by_seat();
    let other = *others.iter().find(|p| **p != owner).unwrap();
    let effects = dispatch_for_events(
        &mut tx,
        &[DomainEvent::TurnStateChanged {
            current: other,
            phase: TurnPhaseTag::Play,
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    assert!(
        effects.is_empty(),
        "owner_is_target must suppress other-player phases"
    );
}
