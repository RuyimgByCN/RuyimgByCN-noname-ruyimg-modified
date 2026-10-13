//! O2 全量合并第二批（2026-10-14）：家族 dcmoukui 的 1 个独立测试文件逐字包裹为模块（小家族/名称冲突/注册表冲突全部纳入——mod 隔离下无碰撞）。证据：docs/operations/evidence/2026-10-14-o2-full-merge-batch1/README.md。

mod original_dcmoukui_machine {
//! 第二百六十一批：dcmoukui 谋溃状态机行为测试（ABI 711 / tag 734 /
//! Cont 312）。链路：杀指定目标（sha first target）→ optional 确认 →
//! Custom [1,2] 可双选：摸 1／弃目标一张手牌。未命中追罚面留 slice 2。

use engine_api::{
    CardId, ChoiceId, Command, CommandKind, CommandResult, DomainEvent, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{GameState, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const MOUKUI: u32 = 2_559_477_796;
const MOUKUI_SRC: &str =
    include_str!("../../../content/original/ported/sp2/dcmoukui.skill");

fn setup(seed: u64) -> (GameState, PlayerId, PlayerId) {
    let compiled = content_compiler::compile_source_named("dcmoukui-b261", MOUKUI_SRC).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 2, seed, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&compiled).unwrap();
    let players = tx.state().players.alive_by_seat();
    let owner = players[0];
    let target = players[1];
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(MOUKUI),
    });
    run_effects(&mut tx, effects).unwrap();
    (tx.commit().state, owner, target)
}

fn sha_card(state: &GameState) -> CardId {
    let draw = state.zones.find(ZoneKind::DrawPile, PlayerId::NONE).unwrap();
    *state
        .zones
        .get(draw)
        .unwrap()
        .cards
        .iter()
        .rev()
        .find(|card| {
            state
                .cards
                .get(**card)
                .is_some_and(|c| c.kind == engine_core::state::CardKind::Sha)
        })
        .expect("no sha in draw pile")
}

fn hand_of(state: &GameState, player: PlayerId) -> Vec<CardId> {
    let hand = state.zones.find(ZoneKind::Hand, player).unwrap();
    state.zones.get(hand).unwrap().cards.clone()
}

fn resolve(
    state: GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    cancelled: bool,
    key: u64,
) -> GameState {
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    let revision = state.revision;
    let (next, result) = apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections,
                cancelled,
            },
        },
    );
    assert!(
        matches!(result.result, CommandResult::Accepted { .. }),
        "resolve rejected {key}: {:?}",
        result.result
    );
    next
}

fn fire_sha(state: GameState, owner: PlayerId, target: PlayerId, sha: CardId) -> (GameState, bool) {
    let mut tx = Transaction::begin(state);
    let outcome = match dispatch_for_events(
        &mut tx,
        &[DomainEvent::CardUsed {
            player: owner,
            card: sha,
            targets: vec![target],
        }],
        &mut TriggerFireRecord::new(),
    ) {
        Err(EffectError::ChoiceRequested) => Err(EffectError::ChoiceRequested),
        Ok(effects) => engine_core::effect::run_effects(&mut tx, effects).map(|_| ()),
        Err(other) => panic!("unexpected: {other:?}"),
    };
    let opened = matches!(outcome, Err(EffectError::ChoiceRequested));
    (tx.commit().state, opened)
}
}
