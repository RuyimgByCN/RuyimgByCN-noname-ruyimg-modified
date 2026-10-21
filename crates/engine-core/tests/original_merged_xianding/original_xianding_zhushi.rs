//! 第一百八十五批：xianding「助势」（曹髦 zhushi）主公技零新原语重写 + 新过滤器。
//! 上游：global recoverEnd；其他魏势力角色于其回合内回复体力后，其可令你摸一张
//! 牌（usable 1）。实现：新过滤器 owner_alive_other_wei_recovery_own_phase_once_
//! per_turn（六点注册含 turn_limit 名单，零 ABI）→ optional → draw_cards(self, 1)。
//! 近似（登记债）：确认方为回复角色 → 拥有者 optional；zhuSkill 身份门未建模。

use engine_api::{
    Command, CommandKind, CommandResult, DomainEvent, ExpectedRevision, IdempotencyKey, PlayerId,
    SkillDefId,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::GameState;
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const ZHUSHI: u32 = 3_044_650_156;
const CAOMAO: u32 = 2_786_357_147;
const SOURCE: &str = include_str!("../../../../content/original/ported/xianding/zhushi.skill");
const XIANDING: &[u8] =
    include_bytes!("../../../../content/original/ported/xianding/xianding.npack.json");

fn setup(seed: u64) -> (GameState, PlayerId, PlayerId) {
    let compiled = content_compiler::compile_source_named("zhushi-test", SOURCE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 2, seed, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content)
        .load_package(&compiled)
        .unwrap();
    let players = tx.state().players.alive_by_seat();
    let owner = players[0];
    let other = players[1];
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(ZHUSHI),
    });
    run_effects(&mut tx, effects).unwrap();
    let state = tx.commit().state;
    // 事件角色为魏势力且处于其自己的回合。
    let mut tx = Transaction::begin(state);
    tx.state_mut().group_overrides.insert(other, "wei".into());
    tx.state_mut().turn.current = other;
    (tx.commit().state, owner, other)
}

fn wei_recovers(state: GameState, other: PlayerId) -> (GameState, bool) {
    let mut tx = Transaction::begin(state);
    let confirmed = match dispatch_for_events(
        &mut tx,
        &[DomainEvent::HpRecovered {
            source: PlayerId::NONE,
            player: other,
            amount: 1,
        }],
        &mut TriggerFireRecord::new(),
    ) {
        Err(EffectError::ChoiceRequested) => true,
        Ok(effects) => {
            run_effects(&mut tx, effects).unwrap();
            false
        }
        Err(other) => panic!("unexpected dispatch error: {other:?}"),
    };
    (tx.commit().state, confirmed)
}

fn resolve(state: GameState, owner: PlayerId, key: u64) -> GameState {
    let choice_id = engine_api::ChoiceId::new(state.resolution.pending_choice_id);
    let revision = state.revision;
    let (next, result) = apply_command(
        state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections: vec![1],
                cancelled: false,
            },
        },
    );
    assert!(
        matches!(result.result, CommandResult::Accepted { .. }),
        "confirm resolve rejected: {:?}",
        result.result
    );
    next
}

#[test]
fn package_publishes_zhushi_contract() {
    let package = content_schema::ContentPackage::from_json_bytes(XIANDING).unwrap();
    assert!(package.manifest.rule_vm_abi_version >= 681);
    let trigger = package
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == ZHUSHI)
        .unwrap();
    assert_eq!(trigger.event, "hp_recovered");
    assert_eq!(
        trigger.filter,
        "owner_alive_other_wei_recovery_own_phase_once_per_turn"
    );
    assert!(trigger.optional);
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == CAOMAO)
        .expect("caomao in roster");
    assert!(character.skills.contains(&ZHUSHI));
}

#[test]
fn wei_recovery_in_own_phase_lets_owner_draw_once_per_turn() {
    let (state, owner, other) = setup(20_260_910_01);
    let hand_before = state.hand_count(owner);

    let (state, confirmed) = wei_recovers(state, other);
    assert!(confirmed, "optional confirm must open");
    assert!(state.has_pending_choice());
    let state = resolve(state, owner, 20_260_910_11);
    assert_eq!(state.hand_count(owner), hand_before + 1, "owner draws one");

    // 每回合限一次：同回合再次回复不触发。
    let (state, confirmed) = wei_recovers(state, other);
    assert!(!confirmed, "turn limit must suppress the second fire");
    assert!(!state.has_pending_choice());
    assert_eq!(state.hand_count(owner), hand_before + 1);
}

#[test]
fn non_wei_recovery_does_not_trigger() {
    let (state, owner, other) = setup(20_260_910_02);
    let mut tx = Transaction::begin(state);
    tx.state_mut().group_overrides.insert(other, "wu".into());
    let state = tx.commit().state;
    let (state, confirmed) = wei_recovers(state, other);
    assert!(
        !confirmed && !state.has_pending_choice(),
        "wu must not fire"
    );
    let _ = owner;
}

#[test]
fn recovery_outside_own_phase_does_not_trigger() {
    let (state, owner, other) = setup(20_260_910_03);
    // 当前回合角色切回 owner：事件角色（other）不再处于自己的回合。
    let mut tx = Transaction::begin(state);
    tx.state_mut().turn.current = owner;
    let state = tx.commit().state;
    let (_, confirmed) = wei_recovers(state, other);
    assert!(
        !confirmed,
        "recovery outside the recovering player's own phase must not fire"
    );
}
