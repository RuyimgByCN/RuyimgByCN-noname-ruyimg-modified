//! 第一百五十批：sp 庞德（SP）「虚待」（spolxudai）零新原语真实语义重写。
//! 上游：限定技，当你响应牌或受到伤害后，你可以令一名其他角色获得〖煮酒〗。
//! 实现：card_responded（owner_is_source）/ damage（owner_is_target）双臂 +
//! choose_other_add_skill(self, 煮酒def) + remove_skill(self_skill)（限定技自移除）。

use engine_api::{
    CardId, ChoiceId, Command, CommandKind, CommandResult, DomainEvent, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{GameState, Phase};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const SPOLXUDAI: u32 = 3_686_006_799;
const SPOLZHUJIU: u32 = 3_397_135_295;
const SPOLXUDAI_SRC: &str = include_str!("../../../../content/original/ported/sp/spolxudai.skill");

fn bootstrap() -> (GameState, PlayerId, PlayerId) {
    let compiled = content_compiler::compile_source_named("spolxudai", SPOLXUDAI_SRC).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 2, u64::from(SPOLXUDAI), "identity-standard").unwrap();
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
        skill_def: SkillDefId::new(SPOLXUDAI),
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

#[test]
fn xudai_damage_arm_grants_zhujiu_and_removes_itself() {
    let (state, owner, other) = bootstrap();
    // 受到 1 点伤害 → 确认 → 选 other → other 获得煮酒、虚待自移除。
    let mut tx = Transaction::begin(state);
    match run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source: other,
            target: owner,
            amount: 1,
            is_lose_hp: false,
            nature: 0,
        }],
    ) {
        Ok(_) => panic!("expected the xudai optional confirm to open"),
        Err(EffectError::ChoiceRequested) => {}
        Err(err) => panic!("unexpected error: {err:?}"),
    }
    let state = tx.commit().state;
    let state = resolve(state, owner, vec![1], 10);
    // 目标选择：options 为其他存活角色，选 other。
    assert!(state.has_pending_choice(), "target selection must open");
    let state = resolve(state, owner, vec![other.raw()], 11);

    assert!(
        has_skill_def(&state, other, SPOLZHUJIU),
        "target must gain 煮酒"
    );
    assert!(
        !has_skill_def(&state, owner, SPOLXUDAI),
        "xudai must remove itself (limited)"
    );

    // 一次性：再次受伤不再触发。
    let mut tx = Transaction::begin(state);
    let opened = run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source: other,
            target: owner,
            amount: 1,
            is_lose_hp: false,
            nature: 0,
        }],
    );
    assert!(
        !matches!(opened, Err(EffectError::ChoiceRequested)),
        "xudai must not fire twice"
    );
}

#[test]
fn xudai_respond_arm_opens_confirm() {
    let (state, owner, other) = bootstrap();
    // 合成响应事件（owner 为响应者）→ 虚待确认打开。
    let mut tx = Transaction::begin(state);
    let opened = dispatch_for_events(
        &mut tx,
        &[DomainEvent::CardResponded {
            player: owner,
            respond_to: other,
            card: CardId::NONE,
            response_kind: 0,
        }],
        &mut TriggerFireRecord::new(),
    );
    assert!(
        matches!(opened, Err(EffectError::ChoiceRequested)),
        "respond arm must open the confirm"
    );
}
