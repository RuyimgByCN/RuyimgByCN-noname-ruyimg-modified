//! 第一百五十三批：KEY 往人「鸦翔」（yukito_yaxiang）补授臂行为测试。
//! 上游：其他角色濒死时变身为乌鸦，令其回复至 3 体力并弃置判定区牌，获得〖终愿〗。
//! 实现（批 153 补授臂）：回复至 3 + 弃判定区 + add_skill(event_player, 终愿def) +
//! remove_skill(self_skill)。近似：不变身。

use engine_api::{
    ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey, PlayerId,
    SkillDefId,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{GameState, Phase};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const YAXIANG: u32 = 2_428_947_588;
const ZHONGYUAN: u32 = 2_446_245_954;
const YAXIANG_SRC: &str =
    include_str!("../../../../content/original/ported/key/yukito_yaxiang.skill");

fn bootstrap() -> (GameState, PlayerId, PlayerId) {
    let compiled = content_compiler::compile_source_named("yukito_yaxiang", YAXIANG_SRC).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 2, u64::from(YAXIANG), "identity-standard").unwrap();
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
        skill_def: SkillDefId::new(YAXIANG),
    });
    run_effects(&mut tx, effects).unwrap();
    let state = tx.commit().state;
    (state, owner, other)
}

fn resolve(state: GameState, actor: PlayerId, selections: Vec<u32>, key: u64) -> GameState {
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    let (state, result) = apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections,
                cancelled: false,
            },
        },
    );
    assert!(
        matches!(result.result, CommandResult::Accepted { .. }),
        "resolve rejected: {:?}",
        result.result
    );
    state
}

fn has_skill_def(state: &GameState, player: PlayerId, def: u32) -> bool {
    state
        .players
        .get(player)
        .unwrap()
        .skills
        .iter()
        .any(|skill| {
            state
                .skills
                .get(*skill)
                .is_some_and(|instance| instance.def.raw() == def)
        })
}

fn hp_of(state: &GameState, player: PlayerId) -> i32 {
    state.players.get(player).unwrap().hp
}

#[test]
fn yaxiang_recovers_equips_grant_and_removes_itself() {
    let (state, owner, other) = bootstrap();
    // other 濒死（hp 置 0，进入 dying 事件由合成派发驱动）。
    let mut tx = Transaction::begin(state);
    tx.state_mut().players.get_mut(other).unwrap().hp = 0;
    let opened = dispatch_for_events_for_test(&mut tx, owner, other);
    assert!(
        matches!(opened, Err(EffectError::ChoiceRequested)),
        "yaxiang optional confirm must open on other's dying"
    );
    let state = tx.commit().state;
    let state = resolve(state, owner, vec![1], 10);

    // 回复至 3、获得终愿、鸦翔自卸。
    assert_eq!(hp_of(&state, other), 3, "target must recover to 3");
    assert!(
        has_skill_def(&state, other, ZHONGYUAN),
        "target must gain 终愿"
    );
    assert!(
        !has_skill_def(&state, owner, YAXIANG),
        "yaxiang must remove itself (limited)"
    );
}

#[test]
fn yaxiang_not_triggered_by_owner_dying() {
    let (state, owner, _other) = bootstrap();
    let mut tx = Transaction::begin(state);
    tx.state_mut().players.get_mut(owner).unwrap().hp = 0;
    let opened = dispatch_for_events_for_test(&mut tx, owner, owner);
    assert!(
        !matches!(opened, Err(EffectError::ChoiceRequested)),
        "owner's own dying must not trigger (owner_alive_not_target)"
    );
}

/// 合成濒死事件（event_player = dying 玩家）驱动鸦翔触发链。
fn dispatch_for_events_for_test(
    tx: &mut Transaction,
    _owner: PlayerId,
    dying: PlayerId,
) -> Result<Vec<EffectRequest>, EffectError> {
    engine_core::trigger::dispatch_for_events(
        tx,
        &[engine_api::DomainEvent::EnteredDying {
            player: dying,
            source: PlayerId::NONE,
        }],
        &mut engine_core::trigger::TriggerFireRecord::new(),
    )
}
